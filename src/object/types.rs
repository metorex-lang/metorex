// The 8-byte runtime value, and the view of its parts that code matches on

use crate::class::Class;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use super::{Binding, BlockStatement, CompiledFunction, Exception, Instance, Method, ObjectHash};

const FALSE: u64 = 0x00;
const NIL: u64 = 0x08;
const TRUE: u64 = 0x14;
const FIXNUM_MAX: i64 = i64::MAX >> 1;
const FIXNUM_MIN: i64 = i64::MIN >> 1;
/// The flonum CRuby uses for 0.0, whose bits would otherwise read as false.
const FLONUM_ZERO: u64 = 0x8000_0000_0000_0002;
/// The bits of 1.0 rotated, which would read as `FLONUM_ZERO`.
const FLONUM_UNUSED: u64 = 0x3000_0000_0000_0000;
/// The low byte of a static Symbol, CRuby's `RUBY_SYMBOL_FLAG`, under its
/// id shifted left by `SYMBOL_SHIFT`.
const SYMBOL_FLAG: u64 = 0x0c;
const SYMBOL_SHIFT: u32 = 8;

/// Every Symbol's name, by id. A name is kept for the life of the thread,
/// so a Symbol's id stays valid wherever its bits are copied.
#[derive(Default)]
struct Symbols {
    names: Vec<&'static Rc<crate::object::StringValue>>,
    ids: std::collections::HashMap<(String, String), u64>,
}

thread_local! {
    static SYMBOLS: RefCell<Symbols> = RefCell::new(Symbols::default());
}

/// The id of the Symbol named `name`, giving it one the first time.
fn symbol_id(name: Rc<crate::object::StringValue>) -> u64 {
    let key = (name.as_str().to_string(), name.encoding_name());
    SYMBOLS.with(|symbols| {
        let mut symbols = symbols.borrow_mut();
        if let Some(id) = symbols.ids.get(&key) {
            return *id;
        }
        let id = symbols.names.len() as u64;
        symbols.names.push(Box::leak(Box::new(name)));
        symbols.ids.insert(key, id);
        id
    })
}

fn symbol_name(id: u64) -> &'static Rc<crate::object::StringValue> {
    SYMBOLS.with(|symbols| symbols.borrow().names[id as usize])
}

/// A runtime value in 8 bytes, laid out as a C extension's `VALUE` is: an
/// Integer that fits in 63 bits shifted left with its low bit set, a Float
/// in the common exponent range as a flonum tagged `0b10`, `false`, `nil`,
/// and `true` as constants, a Symbol as its id over `SYMBOL_FLAG`, and
/// anything else a pointer to a `Body` held by an `Rc`.
pub struct Object {
    bits: u64,
    /// A value holds `Rc`s, so it stays on the thread that made it.
    _single_threaded: PhantomData<Rc<()>>,
}

/// What a pointer value points at. Its alignment keeps a pointer's low
/// three bits clear.
enum Body {
    /// An Integer that fits in an i64 and not in 63 bits.
    Int(i64),
    BigInt(Rc<num_bigint::BigInt>),
    /// A Float whose exponent is outside the flonum range, or -0.0.
    Float(f64),
    String(Rc<crate::object::StringValue>),
    Array(Rc<RefCell<Vec<Object>>>),
    Dict(Rc<RefCell<IndexMap<String, Object>>>),
    Instance(Rc<RefCell<Instance>>),
    Class(Rc<Class>),
    Module(Rc<Class>),
    Method(Rc<Method>),
    Block(Rc<BlockStatement>),
    Exception(Rc<RefCell<Exception>>),
    Set(Rc<RefCell<indexmap::IndexSet<ObjectHash>>>),
    Result(Result<Box<Object>, Box<Object>>),
    NativeFunction(String),
    Range {
        start: Box<Object>,
        end: Box<Object>,
        exclusive: bool,
        mark: Rc<()>,
    },
    Binding(Rc<Binding>),
    CompiledFunction(Rc<CompiledFunction>),
    Regex(Rc<String>, Rc<String>),
}

/// A value's parts, borrowed from it, for matching on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View<'a> {
    Nil,
    Bool(bool),
    Int(i64),
    /// An integer too large for `Int`. Ruby draws no line between the two,
    /// so every result that fits in an i64 is normalized back to `Int` and
    /// this only ever holds a value that does not.
    BigInt(&'a Rc<num_bigint::BigInt>),
    Float(f64),
    /// The text, and the encoding it says it is in.
    String(&'a Rc<crate::object::StringValue>),
    /// A Symbol carries its name as a String carries its text.
    Symbol(&'a Rc<crate::object::StringValue>),
    Array(&'a Rc<RefCell<Vec<Object>>>),
    /// A Hash, backed by an `IndexMap` so entries iterate in insertion order.
    Dict(&'a Rc<RefCell<IndexMap<String, Object>>>),
    Instance(&'a Rc<RefCell<Instance>>),
    Class(&'a Rc<Class>),
    Module(&'a Rc<Class>),
    /// A Method object, bound or unbound.
    Method(&'a Rc<Method>),
    /// A block, proc, or lambda.
    Block(&'a Rc<BlockStatement>),
    Exception(&'a Rc<RefCell<Exception>>),
    Set(&'a Rc<RefCell<indexmap::IndexSet<ObjectHash>>>),
    Result(&'a Result<Box<Object>, Box<Object>>),
    /// A built-in function implemented in Rust, by name.
    NativeFunction(&'a String),
    /// A Range. The mark is what tells two ranges over the same values
    /// apart, since Ruby answers `equal?` for them as it does for any two
    /// objects.
    Range {
        start: &'a Object,
        end: &'a Object,
        exclusive: bool,
        mark: &'a Rc<()>,
    },
    /// A namespace with captured variables.
    Binding(&'a Rc<Binding>),
    /// A function compiled to bytecode.
    CompiledFunction(&'a Rc<CompiledFunction>),
    /// A regular expression: its pattern and its flags.
    Regex(&'a Rc<String>, &'a Rc<String>),
}

#[allow(non_upper_case_globals, non_snake_case)]
impl Object {
    pub const Nil: Object = Object::from_bits(NIL);

    const fn from_bits(bits: u64) -> Object {
        Object {
            bits,
            _single_threaded: PhantomData,
        }
    }

    fn on_heap(body: Body) -> Object {
        Object::from_bits(Rc::into_raw(Rc::new(body)) as u64)
    }

    fn is_pointer(&self) -> bool {
        self.bits & 7 == 0 && self.bits > NIL
    }

    fn body(&self) -> Option<&Body> {
        if !self.is_pointer() {
            return None;
        }
        // SAFETY: a pointer value was made by `on_heap` from `Rc::into_raw`
        // and holds one of the `Rc`'s counts until it is dropped.
        Some(unsafe { &*(self.bits as *const Body) })
    }

    /// The value's bits, as a C extension's `VALUE` would read them.
    pub fn bits(&self) -> u64 {
        self.bits
    }

    pub fn Bool(value: bool) -> Object {
        Object::from_bits(if value { TRUE } else { FALSE })
    }

    pub fn Int(number: i64) -> Object {
        if (FIXNUM_MIN..=FIXNUM_MAX).contains(&number) {
            Object::from_bits(((number as u64) << 1) | 1)
        } else {
            Object::on_heap(Body::Int(number))
        }
    }

    pub fn BigInt(number: Rc<num_bigint::BigInt>) -> Object {
        Object::on_heap(Body::BigInt(number))
    }

    pub fn Float(number: f64) -> Object {
        let bits = number.to_bits();
        if bits == 0 {
            return Object::from_bits(FLONUM_ZERO);
        }
        let top = (bits >> 60) & 7;
        if bits != FLONUM_UNUSED && (top.wrapping_sub(3) & !1) == 0 {
            return Object::from_bits((bits.rotate_left(3) & !1) | 2);
        }
        Object::on_heap(Body::Float(number))
    }

    pub fn String(text: Rc<crate::object::StringValue>) -> Object {
        Object::on_heap(Body::String(text))
    }

    pub fn Symbol(name: Rc<crate::object::StringValue>) -> Object {
        Object::from_bits((symbol_id(name) << SYMBOL_SHIFT) | SYMBOL_FLAG)
    }

    pub fn Array(elements: Rc<RefCell<Vec<Object>>>) -> Object {
        Object::on_heap(Body::Array(elements))
    }

    pub fn Dict(entries: Rc<RefCell<IndexMap<String, Object>>>) -> Object {
        Object::on_heap(Body::Dict(entries))
    }

    pub fn Instance(instance: Rc<RefCell<Instance>>) -> Object {
        Object::on_heap(Body::Instance(instance))
    }

    pub fn Class(class: Rc<Class>) -> Object {
        Object::on_heap(Body::Class(class))
    }

    pub fn Module(module: Rc<Class>) -> Object {
        Object::on_heap(Body::Module(module))
    }

    pub fn Method(method: Rc<Method>) -> Object {
        Object::on_heap(Body::Method(method))
    }

    pub fn Block(block: Rc<BlockStatement>) -> Object {
        Object::on_heap(Body::Block(block))
    }

    pub fn Exception(exception: Rc<RefCell<Exception>>) -> Object {
        Object::on_heap(Body::Exception(exception))
    }

    pub fn Set(members: Rc<RefCell<indexmap::IndexSet<ObjectHash>>>) -> Object {
        Object::on_heap(Body::Set(members))
    }

    pub fn Result(result: Result<Box<Object>, Box<Object>>) -> Object {
        Object::on_heap(Body::Result(result))
    }

    pub fn NativeFunction(name: String) -> Object {
        Object::on_heap(Body::NativeFunction(name))
    }

    pub fn Range(start: Box<Object>, end: Box<Object>, exclusive: bool, mark: Rc<()>) -> Object {
        Object::on_heap(Body::Range {
            start,
            end,
            exclusive,
            mark,
        })
    }

    pub fn Binding(binding: Rc<Binding>) -> Object {
        Object::on_heap(Body::Binding(binding))
    }

    pub fn CompiledFunction(function: Rc<CompiledFunction>) -> Object {
        Object::on_heap(Body::CompiledFunction(function))
    }

    pub fn Regex(pattern: Rc<String>, flags: Rc<String>) -> Object {
        Object::on_heap(Body::Regex(pattern, flags))
    }

    /// The value's parts, for matching on.
    #[inline]
    pub fn view(&self) -> View<'_> {
        let bits = self.bits;
        if bits & 1 == 1 {
            return View::Int((bits as i64) >> 1);
        }
        if bits & 3 == 2 {
            if bits == FLONUM_ZERO {
                return View::Float(0.0);
            }
            let sign = bits >> 63;
            let restored = ((2 - sign) | (bits & !3)).rotate_right(3);
            return View::Float(f64::from_bits(restored));
        }
        let Some(body) = self.body() else {
            return match bits {
                FALSE => View::Bool(false),
                TRUE => View::Bool(true),
                NIL => View::Nil,
                _ => View::Symbol(symbol_name(bits >> SYMBOL_SHIFT)),
            };
        };
        match body {
            Body::Int(number) => View::Int(*number),
            Body::BigInt(number) => View::BigInt(number),
            Body::Float(number) => View::Float(*number),
            Body::String(text) => View::String(text),
            Body::Array(elements) => View::Array(elements),
            Body::Dict(entries) => View::Dict(entries),
            Body::Instance(instance) => View::Instance(instance),
            Body::Class(class) => View::Class(class),
            Body::Module(module) => View::Module(module),
            Body::Method(method) => View::Method(method),
            Body::Block(block) => View::Block(block),
            Body::Exception(exception) => View::Exception(exception),
            Body::Set(members) => View::Set(members),
            Body::Result(result) => View::Result(result),
            Body::NativeFunction(name) => View::NativeFunction(name),
            Body::Range {
                start,
                end,
                exclusive,
                mark,
            } => View::Range {
                start,
                end,
                exclusive: *exclusive,
                mark,
            },
            Body::Binding(binding) => View::Binding(binding),
            Body::CompiledFunction(function) => View::CompiledFunction(function),
            Body::Regex(pattern, flags) => View::Regex(pattern, flags),
        }
    }

    /// How many places hold this value: its copies, which share one cell,
    /// and whatever else holds the parts the cell holds, such as another
    /// cell made for the same object. A value held inline answers 1.
    pub fn references(&self) -> usize {
        if !self.is_pointer() {
            return 1;
        }
        // SAFETY: see `body`. The `Rc` is only read, never dropped.
        let cell = std::mem::ManuallyDrop::new(unsafe { Rc::from_raw(self.bits as *const Body) });
        let parts = match self.view() {
            View::BigInt(held) => Rc::strong_count(held),
            View::String(held) => Rc::strong_count(held),
            View::Array(held) => Rc::strong_count(held),
            View::Dict(held) => Rc::strong_count(held),
            View::Instance(held) => Rc::strong_count(held),
            View::Class(held) | View::Module(held) => Rc::strong_count(held),
            View::Method(held) => Rc::strong_count(held),
            View::Block(held) => Rc::strong_count(held),
            View::Exception(held) => Rc::strong_count(held),
            View::Set(held) => Rc::strong_count(held),
            View::Binding(held) => Rc::strong_count(held),
            View::CompiledFunction(held) => Rc::strong_count(held),
            View::Regex(pattern, _) => Rc::strong_count(pattern),
            View::Range { mark, .. } => Rc::strong_count(mark),
            _ => 1,
        };
        Rc::strong_count(&cell) + parts - 1
    }

    pub fn is_nil(&self) -> bool {
        self.bits == NIL
    }

    /// Get the type name of this object
    pub fn type_name(&self) -> &'static str {
        self.view().type_name()
    }
}

impl Clone for Object {
    #[inline]
    fn clone(&self) -> Object {
        if self.is_pointer() {
            // SAFETY: see `body`.
            unsafe { Rc::increment_strong_count(self.bits as *const Body) };
        }
        Object::from_bits(self.bits)
    }
}

impl Drop for Object {
    #[inline]
    fn drop(&mut self) {
        if self.is_pointer() {
            // SAFETY: see `body`. This value's count is given back once.
            unsafe { Rc::decrement_strong_count(self.bits as *const Body) };
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Object) -> bool {
        self.view() == other.view()
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.view(), formatter)
    }
}

/// `view()` for an optional value, so a match on `Some(View::...)` reads
/// the value inside.
pub trait OptionView<'a> {
    fn view(self) -> Option<View<'a>>;
}

impl<'a> OptionView<'a> for Option<&'a Object> {
    fn view(self) -> Option<View<'a>> {
        self.map(Object::view)
    }
}

impl<'a> OptionView<'a> for Option<&'a mut Object> {
    fn view(self) -> Option<View<'a>> {
        self.map(|object| -> &'a Object { object })
            .map(Object::view)
    }
}

impl<'a> OptionView<'a> for &'a Option<Object> {
    fn view(self) -> Option<View<'a>> {
        self.as_ref().map(Object::view)
    }
}

impl<'a> OptionView<'a> for &'a mut Option<Object> {
    fn view(self) -> Option<View<'a>> {
        self.as_ref().map(Object::view)
    }
}

impl<'a> View<'a> {
    /// The value these parts were read from. A heap value is given a new
    /// pointer to the same parts, so it is the same Ruby object.
    pub fn to_object(self) -> Object {
        match self {
            View::Nil => Object::Nil,
            View::Bool(value) => Object::Bool(value),
            View::Int(number) => Object::Int(number),
            View::BigInt(number) => Object::BigInt(number.clone()),
            View::Float(number) => Object::Float(number),
            View::String(text) => Object::String(text.clone()),
            View::Symbol(name) => Object::Symbol(name.clone()),
            View::Array(elements) => Object::Array(elements.clone()),
            View::Dict(entries) => Object::Dict(entries.clone()),
            View::Instance(instance) => Object::Instance(instance.clone()),
            View::Class(class) => Object::Class(class.clone()),
            View::Module(module) => Object::Module(module.clone()),
            View::Method(method) => Object::Method(method.clone()),
            View::Block(block) => Object::Block(block.clone()),
            View::Exception(exception) => Object::Exception(exception.clone()),
            View::Set(members) => Object::Set(members.clone()),
            View::Result(result) => Object::Result(result.clone()),
            View::NativeFunction(name) => Object::NativeFunction(name.clone()),
            View::Range {
                start,
                end,
                exclusive,
                mark,
            } => Object::Range(
                Box::new(start.clone()),
                Box::new(end.clone()),
                exclusive,
                mark.clone(),
            ),
            View::Binding(binding) => Object::Binding(binding.clone()),
            View::CompiledFunction(function) => Object::CompiledFunction(function.clone()),
            View::Regex(pattern, flags) => Object::Regex(pattern.clone(), flags.clone()),
        }
    }

    /// The name of the kind of value these parts make up.
    pub fn type_name(&self) -> &'static str {
        match self {
            View::Nil => "Nil",
            View::Bool(_) => "Bool",
            View::Int(_) | View::BigInt(_) => "Int",
            View::Float(_) => "Float",
            View::String(_) => "String",
            View::Symbol(_) => "Symbol",
            View::Array(_) => "Array",
            View::Dict(_) => "Dict",
            View::Instance(_) => "Instance",
            View::Class(_) => "Class",
            View::Module(_) => "Module",
            View::Method(_) => "Method",
            View::Block(_) => "Block",
            View::Exception(_) => "Exception",
            View::Set(_) => "Set",
            View::Result(_) => "Result",
            View::NativeFunction(_) => "NativeFunction",
            View::Range { .. } => "Range",
            View::Binding(_) => "Binding",
            View::CompiledFunction(_) => "CompiledFunction",
            View::Regex(_, _) => "Regexp",
        }
    }

    pub fn is_truthy(&self) -> bool {
        !matches!(self, View::Nil | View::Bool(false))
    }

    pub fn as_big_integer(&self) -> Option<num_bigint::BigInt> {
        match self {
            View::Int(value) => Some(num_bigint::BigInt::from(*value)),
            View::BigInt(value) => Some(num_bigint::BigInt::clone(value)),
            _ => None,
        }
    }
}

impl fmt::Display for View<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.to_object(), formatter)
    }
}
