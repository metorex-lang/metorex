// The nodes and tokens of a serialized Prism result, read by native code as
// Prism's C extension builds them for MRI. Prism::Serialize::Loader reads
// the same bytes one `getbyte` call at a time.

use super::prism_node_fields::{NODE_TYPES, SERIALIZATION_VERSION};
use super::*;
use crate::object::{OptionView, View};

/// What a node holds after its flags, as Prism serializes it.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Field {
    Node,
    OptionalNode,
    NodeList,
    Constant,
    OptionalConstant,
    ConstantList,
    Location,
    OptionalLocation,
    String,
    Integer,
    Double,
    Varuint,
    Byte,
}

/// The bytes being read, where the reading is, and what the nodes are built
/// with.
struct Loading {
    bytes: Vec<u8>,
    at: usize,
    source: Object,
    input: Object,
    constant_pool: Object,
    encoding: Object,
    freeze: bool,
    node_classes: Vec<Option<Object>>,
    constants: Vec<Option<Object>>,
    location_class: Option<Object>,
    position: Position,
}

impl Loading {
    fn invalid(&self) -> MetorexError {
        crate::vm::errors::simple_exception("RuntimeError", "Invalid serialization", self.position)
    }

    fn byte(&mut self) -> Result<u8, MetorexError> {
        let byte = *self.bytes.get(self.at).ok_or_else(|| self.invalid())?;
        self.at += 1;
        Ok(byte)
    }

    fn take(&mut self, count: usize) -> Result<&[u8], MetorexError> {
        let end = self
            .at
            .checked_add(count)
            .filter(|end| *end <= self.bytes.len());
        let Some(end) = end else {
            return Err(self.invalid());
        };
        let taken = &self.bytes[self.at..end];
        self.at = end;
        Ok(taken)
    }

    /// An unsigned LEB128 number.
    fn varuint(&mut self) -> Result<u64, MetorexError> {
        let mut value = 0u64;
        let mut shift = 0u32;
        loop {
            let byte = self.byte()?;
            if shift >= u64::BITS {
                return Err(self.invalid());
            }
            value |= u64::from(byte & 0x7f) << shift;
            if byte < 0x80 {
                return Ok(value);
            }
            shift += 7;
        }
    }
}

/// A number read from the bytes as the Integer Ruby makes of it.
fn integer(value: u128) -> Object {
    match i64::try_from(value) {
        Ok(small) => Object::Int(small),
        Err(_) => crate::vm::native_methods::from_big(num_bigint::BigInt::from(value)),
    }
}

/// The arguments `__prism_load_node__` and `__prism_load_tokens__` share: the
/// serialized String and the offset to read from.
fn serialized_at(
    arguments: &[Object],
    position: Position,
) -> Result<(Vec<u8>, usize), MetorexError> {
    let bytes = match arguments.first().view() {
        Some(View::String(text)) => crate::vm::native_methods::string_methods::binary_bytes(text),
        _ => {
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                "wrong argument type (expected String)",
                position,
            ));
        }
    };
    let at = match arguments.get(1).view() {
        Some(View::Int(at)) => usize::try_from(at).unwrap_or(usize::MAX),
        _ => usize::MAX,
    };
    if at > bytes.len() {
        return Err(crate::vm::errors::simple_exception(
            "RuntimeError",
            "Invalid serialization",
            position,
        ));
    }
    Ok((bytes, at))
}

impl VirtualMachine {
    /// The native functions Prism's loader hands its hot paths to, by name.
    pub(crate) fn call_prism_loading_function(
        &mut self,
        name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        match name {
            // The serialization version the node table was written from.
            "__prism_serialization_version__" => Ok(Object::array(
                SERIALIZATION_VERSION
                    .iter()
                    .map(|part| Object::Int(i64::from(*part)))
                    .collect(),
            )),
            // `__prism_load_node__(serialized, offset, source, input,
            // constant_pool, encoding, freeze)` answers the node read from
            // the offset and the offset after it.
            "__prism_load_node__" => {
                let (bytes, at) = serialized_at(arguments, position)?;
                let argument = |at: usize| arguments.get(at).cloned().unwrap_or(Object::Nil);
                let mut loading = Loading {
                    bytes,
                    at,
                    source: argument(2),
                    input: argument(3),
                    constant_pool: argument(4),
                    encoding: argument(5),
                    freeze: argument(6).is_truthy(),
                    node_classes: vec![None; NODE_TYPES.len()],
                    constants: Vec::new(),
                    location_class: None,
                    position,
                };
                let node = self.load_prism_node(&mut loading)?;
                Ok(Object::array(vec![node, Object::Int(loading.at as i64)]))
            }
            // `__prism_load_tokens__(serialized, offset, source, token_types)`
            // answers each token read from the offset, paired with its lexer
            // state, and the offset after them.
            "__prism_load_tokens__" => {
                let (bytes, at) = serialized_at(arguments, position)?;
                let mut loading = Loading {
                    bytes,
                    at,
                    source: arguments.get(2).cloned().unwrap_or(Object::Nil),
                    input: Object::Nil,
                    constant_pool: Object::Nil,
                    encoding: Object::Nil,
                    freeze: false,
                    node_classes: Vec::new(),
                    constants: Vec::new(),
                    location_class: None,
                    position,
                };
                let token_types = arguments.get(3).cloned().unwrap_or(Object::Nil);
                let tokens = self.load_prism_tokens(&mut loading, &token_types)?;
                Ok(Object::array(vec![tokens, Object::Int(loading.at as i64)]))
            }
            _ => Ok(Object::Nil),
        }
    }

    fn prism_class(&mut self, name: &str, position: Position) -> Result<Object, MetorexError> {
        self.resolve_qualified_constant(&format!("Prism::{name}"))
            .ok_or_else(|| {
                crate::vm::errors::simple_exception(
                    "NameError",
                    &format!("uninitialized constant Prism::{name}"),
                    position,
                )
            })
    }

    fn load_prism_tokens(
        &mut self,
        loading: &mut Loading,
        token_types: &Object,
    ) -> Result<Object, MetorexError> {
        let position = loading.position;
        let token_class = self.prism_class("Token", position)?;
        let location_class = self.prism_class("Location", position)?;
        let text = self.send_to_object(loading.source.clone(), "source", Vec::new(), position)?;
        let mut tokens = Vec::new();
        loop {
            let index = loading.varuint()?;
            let kind = match token_types.view() {
                View::Array(listed) => listed.borrow().get(index as usize).cloned(),
                _ => None,
            };
            let Some(kind) = kind else {
                return Err(crate::vm::errors::simple_exception(
                    "IndexError",
                    &format!("index {index} outside of array bounds"),
                    position,
                ));
            };
            if kind.is_nil() {
                break;
            }
            let start = Object::Int(loading.varuint()? as i64);
            let length = Object::Int(loading.varuint()? as i64);
            let lex_state = Object::Int(loading.varuint()? as i64);
            let location = self.send_to_object(
                location_class.clone(),
                "new",
                vec![loading.source.clone(), start.clone(), length.clone()],
                position,
            )?;
            let value =
                self.send_to_object(text.clone(), "byteslice", vec![start, length], position)?;
            if matches!(value.view(), View::Nil) {
                return Err(crate::vm::errors::simple_exception(
                    "RuntimeError",
                    "unhandled exception",
                    position,
                ));
            }
            let token = self.send_to_object(
                token_class.clone(),
                "new",
                vec![loading.source.clone(), kind, value, location],
                position,
            )?;
            tokens.push(Object::array(vec![token, lex_state]));
        }
        Ok(Object::array(tokens))
    }

    fn load_prism_node(&mut self, loading: &mut Loading) -> Result<Object, MetorexError> {
        let position = loading.position;
        let kind = usize::from(loading.byte()?);
        let Some(&(name, skips_length, fields)) =
            kind.checked_sub(1).and_then(|at| NODE_TYPES.get(at))
        else {
            return Err(loading.invalid());
        };
        let node_id = integer(u128::from(loading.varuint()?));
        let location = self.load_prism_location(loading)?;
        if skips_length {
            loading.take(4)?;
        }
        let flags = integer(u128::from(loading.varuint()?));
        let mut arguments = Vec::with_capacity(4 + fields.len());
        arguments.extend([loading.source.clone(), node_id, location, flags]);
        for field in fields {
            let value = self.load_prism_field(loading, *field)?;
            arguments.push(value);
        }
        let class = match loading.node_classes[kind - 1].clone() {
            Some(class) => class,
            None => {
                let class = self.prism_class(name, position)?;
                loading.node_classes[kind - 1] = Some(class.clone());
                class
            }
        };
        let node = self.send_to_object(class, "new", arguments, position)?;
        self.frozen_when_asked(loading, node)
    }

    fn frozen_when_asked(
        &mut self,
        loading: &Loading,
        value: Object,
    ) -> Result<Object, MetorexError> {
        if loading.freeze {
            return self.send_to_object(value, "freeze", Vec::new(), loading.position);
        }
        Ok(value)
    }

    /// A location as `load_location` reads it: its start and length packed
    /// into one Integer, or a frozen Location when the result is frozen.
    fn load_prism_location(&mut self, loading: &mut Loading) -> Result<Object, MetorexError> {
        let start = loading.varuint()?;
        let length = loading.varuint()?;
        if !loading.freeze {
            return Ok(integer((u128::from(start) << 32) | u128::from(length)));
        }
        let class = match loading.location_class.clone() {
            Some(class) => class,
            None => {
                let class = self.prism_class("Location", loading.position)?;
                loading.location_class = Some(class.clone());
                class
            }
        };
        let location = self.send_to_object(
            class,
            "new",
            vec![
                loading.source.clone(),
                integer(u128::from(start)),
                integer(u128::from(length)),
            ],
            loading.position,
        )?;
        self.frozen_when_asked(loading, location)
    }

    fn load_prism_constant(
        &mut self,
        loading: &mut Loading,
        index: u64,
    ) -> Result<Object, MetorexError> {
        let at = (index - 1) as usize;
        if let Some(Some(constant)) = loading.constants.get(at) {
            return Ok(constant.clone());
        }
        let constant = self.send_to_object(
            loading.constant_pool.clone(),
            "get",
            vec![integer(u128::from(index - 1)), loading.encoding.clone()],
            loading.position,
        )?;
        if loading.constants.len() <= at {
            loading.constants.resize(at + 1, None);
        }
        loading.constants[at] = Some(constant.clone());
        Ok(constant)
    }

    fn load_prism_field(
        &mut self,
        loading: &mut Loading,
        field: Field,
    ) -> Result<Object, MetorexError> {
        let position = loading.position;
        match field {
            Field::Node => self.load_prism_node(loading),
            Field::OptionalNode => {
                if loading.bytes.get(loading.at) == Some(&0) {
                    loading.at += 1;
                    return Ok(Object::Nil);
                }
                self.load_prism_node(loading)
            }
            Field::NodeList => {
                let count = loading.varuint()?;
                let mut nodes = Vec::with_capacity(count.min(1 << 16) as usize);
                for _ in 0..count {
                    nodes.push(self.load_prism_node(loading)?);
                }
                self.frozen_when_asked(loading, Object::array(nodes))
            }
            Field::Constant => {
                let index = loading.varuint()?;
                if index == 0 {
                    return Err(loading.invalid());
                }
                self.load_prism_constant(loading, index)
            }
            Field::OptionalConstant => match loading.varuint()? {
                0 => Ok(Object::Nil),
                index => self.load_prism_constant(loading, index),
            },
            Field::ConstantList => {
                let count = loading.varuint()?;
                let mut constants = Vec::with_capacity(count.min(1 << 16) as usize);
                for _ in 0..count {
                    let index = loading.varuint()?;
                    if index == 0 {
                        return Err(loading.invalid());
                    }
                    constants.push(self.load_prism_constant(loading, index)?);
                }
                self.frozen_when_asked(loading, Object::array(constants))
            }
            Field::Location => self.load_prism_location(loading),
            Field::OptionalLocation => {
                if loading.byte()? == 0 {
                    return Ok(Object::Nil);
                }
                self.load_prism_location(loading)
            }
            Field::String => {
                let text = match loading.byte()? {
                    1 => {
                        let start = integer(u128::from(loading.varuint()?));
                        let length = integer(u128::from(loading.varuint()?));
                        self.send_to_object(
                            loading.input.clone(),
                            "byteslice",
                            vec![start, length],
                            position,
                        )?
                    }
                    2 => {
                        let length = loading.varuint()? as usize;
                        let held = loading.take(length)?;
                        crate::vm::native_methods::pack_format::bytes_to_string(held)
                    }
                    other => {
                        return Err(crate::vm::errors::simple_exception(
                            "RuntimeError",
                            &format!("Unknown serialized string type: {other}"),
                            position,
                        ));
                    }
                };
                let text = self.send_to_object(
                    text,
                    "force_encoding",
                    vec![loading.encoding.clone()],
                    position,
                )?;
                self.send_to_object(text, "freeze", Vec::new(), position)
            }
            Field::Integer => {
                let negative = loading.byte()? != 0;
                let length = loading.varuint()?;
                let mut value = num_bigint::BigInt::from(0);
                for index in 0..length {
                    let word = num_bigint::BigInt::from(loading.varuint()?);
                    value |= word << (index * 32);
                }
                if negative {
                    value = -value;
                }
                Ok(crate::vm::native_methods::from_big(value))
            }
            Field::Double => {
                let held = loading.take(8)?;
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(held);
                Ok(Object::Float(f64::from_ne_bytes(bytes)))
            }
            Field::Varuint => Ok(integer(u128::from(loading.varuint()?))),
            Field::Byte => Ok(Object::Int(i64::from(loading.byte()?))),
        }
    }
}
