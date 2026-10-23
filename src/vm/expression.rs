//! Expression evaluation functions for the Metorex VM.
//!
//! This module contains the core logic for evaluating expressions including:
//! - Interpolated strings
//! - Array literals
//! - Dictionary literals
//! - Index operations (array/dictionary access)

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::object::{OptionView, View};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

use super::core::VirtualMachine;
use super::utils::position_to_location;

impl VirtualMachine {
    /// The text a value puts in an interpolation's hole, which is what its
    /// `to_s` answers. Interpolation uses #to_s semantics: a Symbol renders
    /// as its bare name, not the `:name` inspect form.
    pub(crate) fn interpolated_text(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        // A refinement in force says how a value writes itself, whatever kind
        // it is, so it is asked before the kind's own rendering is used.
        let asked = self.any_refinement_active()
            && crate::vm::method_lookup::refinement_target_name(value, self)
                .and_then(|target| self.find_refined_method(&target, "to_s"))
                .is_some();
        if !asked {
            match value.view() {
                View::Symbol(name) => return Ok(name.as_str().to_string()),
                // `nil.to_s` is the empty string, so `"#{nil}"` adds nothing
                // rather than the `nil` inspect form.
                View::Nil => return Ok(String::new()),
                // An object of the program's own is asked for its own `to_s`,
                // which is the text Ruby puts in the hole, and so is an
                // exception, whose `to_s` is its message. A pattern writes
                // itself as the group it stands for, which is what
                // `Regexp#to_s` answers, and an Array or a Hash as its
                // `inspect` form. A Range writes its ends through their own
                // `to_s`.
                View::Instance(_)
                | View::Exception(_)
                | View::Regex(_, _)
                | View::Array(_)
                | View::Dict(_)
                | View::Range { .. } => {}
                _ => return Ok(value.to_string()),
            }
        }
        let written = self.send_to_object(value.clone(), "to_s", Vec::new(), position)?;
        Ok(match written.view() {
            View::String(text) => text.as_str().to_string(),
            // A `to_s` answering something other than a String leaves the
            // value written as Kernel#to_s writes it.
            _ => value.to_string(),
        })
    }
}

/// An interpolated literal as its pieces are added: the text they spell, the
/// encoding of the first piece that held anything outside ASCII and whether
/// that piece stood for bytes rather than characters, and the bytes
/// themselves where any piece stood for bytes.
#[derive(Default)]
pub(crate) struct Interpolation {
    buffer: String,
    raw: Vec<u8>,
    any_bytes: bool,
    carried: Option<(String, bool)>,
    escaped_bytes_written: bool,
    non_ascii_text_written: bool,
}

impl Interpolation {
    /// Add a piece of the literal's own text.
    pub(crate) fn push_text(&mut self, text: &str) {
        self.non_ascii_text_written |= !text.is_ascii();
        // Bytes the literal escaped stand for themselves.
        if let Some(bytes) = crate::file_loader::escaped_source_bytes(text) {
            self.escaped_bytes_written = true;
            self.any_bytes = true;
            self.buffer.extend(bytes.iter().map(|byte| *byte as char));
            self.raw.extend(bytes);
            return;
        }
        self.buffer.push_str(text);
        self.raw.extend(text.as_bytes());
    }

    /// Add the value of a hole, `written` being the text
    /// `interpolated_text` gave for it.
    pub(crate) fn push_value(
        &mut self,
        value: &Object,
        written: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let wrote = self.buffer.len();
        self.buffer.push_str(written);
        // A UTF-16 or UTF-32 piece spells even ASCII text in bytes of its own,
        // so its encoding carries whatever it holds.
        if let View::String(spelled) = value.view()
            && (!spelled.as_str().is_ascii()
                || crate::vm::native_methods::string_methods::wide_encoding(
                    &spelled.encoding_name(),
                )
                .is_some())
        {
            // Two pieces that each hold something outside ASCII have to be
            // written in the same encoding, since the whole can only be
            // written in one of them.
            if let Some((held, _)) = &self.carried
                && *held != spelled.encoding_name()
            {
                let message =
                    crate::vm::native_methods::string_methods::incompatible_encodings_message(
                        held,
                        &spelled.encoding_name(),
                    );
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("Encoding::CompatibilityError", message.clone()),
                    location: position_to_location(position),
                    message,
                });
            }
            if self.carried.is_none() {
                self.carried = Some((spelled.encoding_name(), spelled.holds_bytes()));
            }
        }
        if let View::String(spelled) = value.view()
            && spelled.holds_bytes()
        {
            self.any_bytes = true;
            self.raw
                .extend(crate::vm::native_methods::string_methods::binary_bytes(
                    spelled,
                ));
            return Ok(());
        }
        self.raw.extend(&self.buffer.as_bytes()[wrote..]);
        Ok(())
    }

    /// The String the pieces make.
    pub(crate) fn finish(self, vm: &VirtualMachine) -> Object {
        let Interpolation {
            buffer: text,
            raw,
            any_bytes,
            carried,
            escaped_bytes_written,
            non_ascii_text_written,
        } = self;
        // A piece standing for the bytes it was read from makes the whole
        // string stand for bytes, and every other piece is written out in the
        // bytes its own encoding spells it with.
        if any_bytes {
            let spelled = crate::object::StringValue::binary(raw);
            match &carried {
                Some((encoding, _)) => spelled.set_encoding(encoding.clone()),
                // Bytes the literal itself escaped are written in the
                // encoding its source is written in.
                None if escaped_bytes_written => {
                    spelled.set_encoding(vm.source_literal_encoding().unwrap_or_else(|| {
                        crate::object::string_value::DEFAULT_ENCODING.to_string()
                    }))
                }
                None => {}
            }
            return Object::String(std::rc::Rc::new(spelled));
        }
        let Some((encoding, holds_bytes)) = carried.filter(|_| !non_ascii_text_written) else {
            // Nothing outside ASCII went in, so the whole is written in the
            // encoding its source is written in, the way a plain literal is.
            return match vm.source_literal_encoding().filter(|_| text.is_ascii()) {
                Some(named) => Object::String(std::rc::Rc::new(
                    crate::object::StringValue::with_encoding(text, named),
                )),
                None => Object::string(text),
            };
        };
        let spelled = crate::object::StringValue::with_encoding(text, encoding);
        if holds_bytes {
            spelled.mark_bytes();
        }
        Object::String(std::rc::Rc::new(spelled))
    }
}

impl VirtualMachine {
    /// The values `*value` spreads into: an Array's elements, nothing for
    /// nil, an Array subclass's elements, what `to_a` answers, or the value
    /// alone.
    pub(crate) fn splat_elements(
        &mut self,
        value: Object,
        position: crate::lexer::Position,
    ) -> Result<Vec<Object>, MetorexError> {
        match value.view() {
            View::Array(items) => Ok(items.borrow().clone()),
            View::Nil => Ok(Vec::new()),
            held => {
                match crate::vm::native_methods::subclasses::array_subclass_value(&held.to_object())
                    .view()
                {
                    Some(View::Array(items)) => Ok(items.borrow().clone()),
                    // Anything else that answers `to_a` spreads into what that
                    // gives, which is what `[*"a".."z"]` names.
                    _ => Ok(match self.splat_through_to_a(held.to_object(), position)? {
                        Ok(items) => items,
                        Err(held) => vec![held],
                    }),
                }
            }
        }
    }

    /// What a splat spreads a value into. A value answering `to_a` spreads
    /// into what that gives; anything else stands for itself, which the Err
    /// side carries back.
    pub(crate) fn splat_through_to_a(
        &mut self,
        value: Object,
        position: crate::lexer::Position,
    ) -> Result<Result<Vec<Object>, Object>, MetorexError> {
        if !self.answers_conversion(&value, "to_a", position)? {
            return Ok(Err(value));
        }
        // `to_a` answering nil leaves the value whole, and answering anything
        // else but an Array is refused.
        match self
            .send_to_object(value.clone(), "to_a", vec![], position)?
            .view()
        {
            View::Array(items) => Ok(Ok(items.borrow().clone())),
            View::Nil => Ok(Err(value)),
            answered => {
                let named = self.builtins().class_of(&value).ruby_name();
                let gives = self.builtins().class_of(&answered.to_object()).ruby_name();
                Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &format!("can't convert {named} to Array ({named}#to_a gives {gives})"),
                    position,
                ))
            }
        }
    }

    /// The Hash a `**value` spreads: the value itself, what its `to_hash`
    /// answers, or nothing at all for nil.
    pub(crate) fn hash_for_splat(
        &mut self,
        value: Object,
        position: crate::lexer::Position,
    ) -> Result<Option<SharedPairs>, MetorexError> {
        match value.view() {
            View::Dict(pairs) => Ok(Some(pairs.clone())),
            View::Nil => Ok(None),
            _ if self.lookup_method(&value, "to_hash").is_some() => {
                let class_name = self.conversion_name(&value);
                match self
                    .send_to_object(value.clone(), "to_hash", vec![], position)?
                    .view()
                {
                    View::Dict(pairs) => Ok(Some(pairs.clone())),
                    answered => {
                        let message = format!(
                            "can't convert {} to Hash ({}#to_hash gives {})",
                            class_name,
                            class_name,
                            self.conversion_name(&answered.to_object())
                        );
                        Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &message,
                            position,
                        ))
                    }
                }
            }
            _ => {
                let message = format!(
                    "no implicit conversion of {} into Hash",
                    self.conversion_name(&value)
                );
                Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &message,
                    position,
                ))
            }
        }
    }

    /// Ruby names a key written twice in the same literal, reporting the line
    /// whose value wins. Only a literal key is reported, since two expressions
    /// that happen to answer the same object are not a duplicate as written.
    fn warn_duplicated_key(
        &mut self,
        key: &Object,
        written: &WrittenPair,
    ) -> Result<(), MetorexError> {
        let position = written.key_position;
        let file = self
            .current_source_file
            .clone()
            .or_else(|| {
                self.current_file
                    .as_ref()
                    .map(|path| path.display().to_string())
            })
            .unwrap_or_default();
        if !self
            .reported_duplicate_keys
            .insert((file.clone(), position.line, position.column))
        {
            return Ok(());
        }
        let rendered = crate::vm::native_methods::array_methods::inspect_element(key);
        let message = format!(
            "{}:{}: warning: key {} is duplicated and overwritten on line {}\n",
            file, position.line, rendered, written.value_line
        );
        self.warn_through_warning_module(message, position)
    }

    /// Evaluate indexing operations on arrays and dictionaries.
    /// The number an end of a span names, read through `to_int` when it is
    /// not already an Integer. A nil end names no number at all.
    pub(crate) fn span_end_index(
        &mut self,
        edge: &Object,
        position: Position,
    ) -> Result<Option<i64>, MetorexError> {
        match edge.view() {
            View::Nil => Ok(None),
            View::Int(number) => Ok(Some(number)),
            View::Float(number) => Ok(Some(number as i64)),
            // A number past what a machine word holds names no place in a
            // collection, which Ruby reports as a range rather than a type.
            View::BigInt(_) => Err(crate::vm::errors::simple_exception(
                "RangeError",
                "bignum too big to convert into 'long'",
                position,
            )),
            _ if self.answers_to(edge, "to_int", position)? => {
                match self
                    .send_to_object(edge.clone(), "to_int", vec![], position)?
                    .view()
                {
                    View::Int(number) => Ok(Some(number)),
                    View::Float(number) => Ok(Some(number as i64)),
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn evaluate_index_operation(
        &mut self,
        collection: Object,
        key: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // The environment names its variables in text, so a lookup reads what
        // it was handed as text first.
        if let View::Dict(dict_rc) = collection.view()
            && self.dict_is_environment(dict_rc)
        {
            let held = Object::Dict(Rc::clone(dict_rc));
            return self
                .call_hash_method(&held, "[]", &[key], position)?
                .ok_or_else(|| {
                    MetorexError::runtime_error(
                        "ENV lookup answered nothing",
                        crate::vm::utils::position_to_location(position),
                    )
                });
        }
        match collection.view() {
            View::Array(elements_rc) => match key.view() {
                View::Int(index) => {
                    let elements = elements_rc.borrow();
                    // A negative index counts back from the end, and one past
                    // either end answers nil rather than raising.
                    let offset = if index < 0 {
                        match elements.len().checked_sub(index.unsigned_abs() as usize) {
                            Some(offset) => offset,
                            None => return Ok(Object::Nil),
                        }
                    } else {
                        index as usize
                    };
                    Ok(elements.get(offset).cloned().unwrap_or(Object::Nil))
                }
                // `ary[range]` answers the slice the range covers. A negative
                // bound counts from the end, and a start past the end answers
                // nil the way Ruby's does.
                // A range over Integers is read here, and one over anything
                // else is left to the method, which converts its ends.
                View::Range {
                    ref start,
                    ref end,
                    exclusive,
                    ..
                } if [start, end]
                    .iter()
                    .all(|bound| matches!(bound.view(), View::Int(_) | View::Nil)) =>
                {
                    let elements = elements_rc.borrow();
                    let length = elements.len() as i64;
                    let resolve = |bound: &Object| match bound.view() {
                        View::Int(value) if value < 0 => value + length,
                        View::Int(value) => value,
                        _ => 0,
                    };
                    let from = match start.view() {
                        View::Nil => 0,
                        _ => resolve(start),
                    };
                    let mut to = match end.view() {
                        View::Nil => length,
                        _ => {
                            let resolved = resolve(end);
                            if exclusive { resolved } else { resolved + 1 }
                        }
                    };
                    if from < 0 || from > length {
                        return Ok(Object::Nil);
                    }
                    to = to.clamp(from, length);
                    let slice = elements[from as usize..to as usize].to_vec();
                    drop(elements);
                    Ok(Object::Array(Rc::new(RefCell::new(slice))))
                }
                // Anything else an array reads as a subscript — an object
                // answering `to_int`, an arithmetic sequence, a Range
                // subclass — is settled by the method itself.
                other => {
                    let named = other.type_name().to_string();
                    let held = Object::Array(Rc::clone(elements_rc));
                    match self.call_array_method(
                        &held,
                        "[]",
                        std::slice::from_ref(&key),
                        position,
                    )? {
                        Some(found) => Ok(found),
                        None => Err(MetorexError::type_error(
                            format!("Array index must be an Integer, found {}", named),
                            position_to_location(position),
                        )),
                    }
                }
            },
            View::Dict(dict_rc) => {
                let (fresh, held) = self.hash_locate_key(dict_rc, &key, position)?;
                let key_string = held.unwrap_or(fresh);

                let dict = dict_rc.borrow();
                if let Some(value) = dict.get(&key_string) {
                    Ok(value.clone())
                } else if let Some(default) = dict.get("__MX_DEFAULT__") {
                    // `Hash.new(default)` answers that value for a key it has
                    // no entry for, without storing anything.
                    Ok(default.clone())
                } else if let Some(View::Block(default_proc)) =
                    dict.get("__MX_DEFAULT_PROC__").view()
                {
                    // Auto-vivify: call the default block with (hash, key)
                    // The block typically does h[k] = default_value, which sets
                    // the value directly in the hash.
                    let block = default_proc.clone();
                    drop(dict);
                    let hash_obj = Object::Dict(Rc::clone(dict_rc));
                    let block_result =
                        self.execute_block_callable(&block, vec![hash_obj, key.clone()], position)?;
                    // The block may have set the key directly (h[k] = val), or
                    // returned a value. Check the hash first, fall back to block result.
                    let stored = dict_rc.borrow().get(&key_string).cloned();
                    if let Some(value) = stored {
                        Ok(value)
                    } else {
                        // Ruby keeps nothing for a key the block did not
                        // store, so the next read calls the block again.
                        Ok(block_result)
                    }
                } else {
                    Ok(Object::Nil)
                }
            }

            // Symbol#[] mirrors String#[] on the symbol's name.
            View::Symbol(s) => {
                let as_string = Object::string(s.as_str().to_string());
                self.evaluate_index_operation(as_string, key, position)
            }

            // A String reads its own `[]`, which keeps one reading of the
            // characters for both the index form and the method call.
            View::String(s) => {
                // A Float or an object that names an index arrives as one,
                // and a Range subclass stands for the span it holds.
                let key = match key.view() {
                    View::Int(_) | View::Range { .. } | View::String(_) | View::Regex(_, _) => key,
                    View::Float(number) => Object::Int(number as i64),
                    _ => match crate::vm::native_methods::as_range(&key) {
                        Some(span) => span,
                        None if self.responds_to(&key, "to_int") => {
                            self.send_to_object(key.clone(), "to_int", vec![], position)?
                        }
                        None => key,
                    },
                };
                let made = match key.view() {
                    View::Int(i) => {
                        let as_text =
                            crate::vm::native_methods::string_methods::characters_are_text(s);
                        let chars = if as_text {
                            Vec::new()
                        } else {
                            crate::vm::native_methods::string_methods::character_units(s)
                        };
                        let len = if as_text {
                            s.character_count()
                        } else {
                            chars.len()
                        } as i64;
                        let idx = if i < 0 { len + i } else { i };
                        if idx < 0 || idx >= len {
                            Ok(Object::Nil)
                        } else if as_text {
                            let at = idx as usize;
                            Ok(Object::string(s.character_slice(at, at + 1)))
                        } else {
                            Ok(Object::string(chars[idx as usize].clone()))
                        }
                    }
                    View::Range {
                        start,
                        end,
                        exclusive,
                        ..
                    } => {
                        let as_text =
                            crate::vm::native_methods::string_methods::characters_are_text(s);
                        let chars = if as_text {
                            Vec::new()
                        } else {
                            crate::vm::native_methods::string_methods::character_units(s)
                        };
                        let len = if as_text {
                            s.character_count()
                        } else {
                            chars.len()
                        } as i64;
                        let from = match self.span_end_index(start, position)? {
                            Some(number) => {
                                let placed = if number < 0 { len + number } else { number };
                                if placed < 0 || placed > len {
                                    return Ok(Object::Nil);
                                }
                                placed
                            }
                            None => 0,
                        };
                        let last = match self.span_end_index(end, position)? {
                            Some(number) => {
                                let placed = if number < 0 { len + number } else { number };
                                if exclusive { placed - 1 } else { placed }
                            }
                            None => len - 1,
                        };
                        let count = (last - from + 1).max(0);
                        let stop = (from + count).min(len);
                        let sliced: String = if as_text {
                            s.character_slice(from as usize, stop as usize)
                        } else {
                            chars[from as usize..stop as usize].concat()
                        };
                        Ok(Object::string(sliced))
                    }
                    // `text[other]` answers the other string when it appears,
                    // which is how Ruby looks a substring up.
                    View::String(wanted) => {
                        if s.as_str().contains(&*wanted.as_str()) {
                            Ok(Object::string(wanted.as_str().to_string()))
                        } else {
                            Ok(Object::Nil)
                        }
                    }
                    // `text[pattern]` answers what the pattern matched, and
                    // the match is recorded so `$~` names it afterwards.
                    View::Regex(pattern, flags) => {
                        let (pattern, flags) =
                            (pattern.as_str().to_string(), flags.as_str().to_string());
                        let subject = s.as_str().to_string();
                        let encoding = Some(s.encoding_name());
                        match self.regexp_match_data_in(
                            &pattern, &flags, &subject, 0, encoding, position,
                        )? {
                            Some(found) => self
                                .send_to_object(found, "[]", vec![Object::Int(0)], position)
                                .map(Ok)?,
                            None => Ok(Object::Nil),
                        }
                    }
                    // A number too wide to count characters with names no
                    // place in the string.
                    View::BigInt(_) => Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "bignum too big to convert into 'long'",
                        position,
                    )),
                    // An instance of a String subclass carries its characters
                    // in an instance variable, and names a substring the same
                    // way a plain String does.
                    View::Instance(_) => {
                        match crate::vm::native_methods::string_subclass_value(&key).view() {
                            Some(View::String(wanted)) => {
                                if s.as_str().contains(&*wanted.as_str()) {
                                    Ok(Object::string(wanted.as_str().to_string()))
                                } else {
                                    Ok(Object::Nil)
                                }
                            }
                            _ => Err(crate::vm::errors::integer_conversion_error(&key, position)),
                        }
                    }
                    _ => Err(crate::vm::errors::integer_conversion_error(&key, position)),
                }?;
                // A piece of a string is written in the same encoding, and
                // its characters stand for what the whole one's stood for.
                if let View::String(piece) = made.view()
                    && piece.encoding_name() == crate::object::string_value::DEFAULT_ENCODING
                {
                    piece.set_encoding(s.encoding_name());
                    if s.holds_bytes() {
                        piece.mark_bytes();
                    }
                }
                Ok(made)
            }

            View::Class(_) | View::Module(_) | View::Instance(_) => {
                // A user-defined `[]` wins: on a class or module that is
                // `def self.[]`, stored under the `__class__` prefix.
                if let Some((owner, method)) = self.lookup_method(&collection, "[]")
                    && !method.is_undefined
                {
                    return self.invoke_method(
                        owner,
                        method,
                        collection.clone(),
                        vec![key],
                        position,
                    );
                }
                // Otherwise the native `[]` (Dir[], Hash[], and friends).
                let class = self.builtins().class_of(&collection);
                match self.call_native_method(
                    &class,
                    &collection,
                    "[]",
                    std::slice::from_ref(&key),
                    position,
                )? {
                    Some(val) => Ok(val),
                    // A class that answers what it was not asked for decides
                    // what indexing means, the same as it does for any other
                    // name it carries no method for.
                    None if self
                        .lookup_method(&collection, "method_missing")
                        .is_some_and(|(_, method)| !method.is_undefined) =>
                    {
                        self.send_to_object(
                            collection,
                            "method_missing",
                            vec![Object::symbol("[]"), key],
                            position,
                        )
                    }
                    None => {
                        let wording = self.receiver_wording_for(&collection, position);
                        Err(crate::vm::errors::undefined_method_error_worded(
                            "[]",
                            &collection,
                            std::slice::from_ref(&key),
                            wording,
                            position,
                        ))
                    }
                }
            }
            // Any other receiver, such as an exception whose class defines
            // `[]`, is sent the call.
            _ => self.send_to_object(collection.clone(), "[]", vec![key], position),
        }
    }

    /// The place a flip-flop's state is kept: the flip-flop written at this
    /// place in this piece of code. Two `eval` calls over the same text are
    /// two pieces, so the name the code was read under is part of the key.
    fn flip_flop_site(&self, position: Position) -> (String, usize, usize, usize) {
        (
            self.current_source_file.clone().unwrap_or_default(),
            position.line,
            position.column,
            position.offset,
        )
    }

    /// Whether the flip-flop written at `position` is on. The first time it
    /// is read, a number written as one of its `integer_sides` sides is
    /// warned about in verbose mode: it is compared against the line the
    /// last read left off at, which is rarely what the program meant.
    pub(crate) fn flip_flop_running(&mut self, position: Position, integer_sides: usize) -> bool {
        let site = self.flip_flop_site(position);
        if let Some(running) = self.flip_flops.get(&site) {
            return *running;
        }
        self.flip_flops.insert(site, false);
        if matches!(self.globals().get("VERBOSE").view(), Some(View::Bool(true))) {
            for _ in 0..integer_sides {
                self.emit_warning_to_stderr("warning: integer literal in flip-flop", position);
            }
        }
        false
    }

    /// Turn the flip-flop written at `position` on or off.
    pub(crate) fn set_flip_flop(&mut self, position: Position, on: bool) {
        let site = self.flip_flop_site(position);
        self.flip_flops.insert(site, on);
    }
}

/// Whether a key is written out in the literal itself, which is what Ruby
/// checks for a duplicate rather than what the key evaluates to.
/// The pairs a Hash holds, shared with every reference to it.
type SharedPairs = Rc<RefCell<IndexMap<String, Object>>>;

/// The pairs of a Hash literal with each `**{...}` literal written inside it
/// spread in place, since its keys count as written in the outer literal.
pub(crate) fn flattened_entries(
    entries: &[(Expression, Expression)],
) -> Vec<(&Expression, &Expression)> {
    let mut flat = Vec::with_capacity(entries.len());
    for (key, value) in entries {
        if let Expression::KeywordSplat { expression, .. } = key
            && let Expression::Dictionary { entries: inner, .. } = expression.as_ref()
        {
            flat.extend(flattened_entries(inner));
            continue;
        }
        flat.push((key, value));
    }
    flat
}

/// Where a pair of a Hash literal was written, which the warning for a key
/// written twice names.
pub(crate) struct WrittenPair {
    pub(crate) key_position: Position,
    pub(crate) value_line: usize,
    /// Whether the key was written as a literal, the only keys the warning
    /// is given for.
    pub(crate) literal_key: bool,
}

/// A Hash literal as its pairs are added.
#[derive(Default)]
pub(crate) struct HashLiteral {
    map: IndexMap<String, Object>,
    key_objs: IndexMap<String, Object>,
    // A key that came in through `**held` was not written twice, so writing
    // it again is not the duplicate Ruby reports.
    spread_names: std::collections::HashSet<String>,
}

/// A key of a Hash literal, ready for its value.
pub(crate) struct HashKey {
    value: Object,
    slot: String,
}

impl HashLiteral {
    /// Copy every pair `spread` carries into the hash, as `**spread` does.
    pub(crate) fn spread(
        &mut self,
        vm: &mut VirtualMachine,
        spread: Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let Some(pairs) = vm.hash_for_splat(spread, position)? else {
            return Ok(());
        };
        let pairs = pairs.borrow();
        if let Some(View::Dict(objects)) = pairs
            .get(crate::vm::native_methods::hash_methods::KEY_OBJECTS_KEY)
            .view()
        {
            for (slot, held) in objects.borrow().iter() {
                self.key_objs.insert(slot.clone(), held.clone());
            }
        }
        for (name, held) in pairs.iter() {
            if crate::vm::native_methods::hash_methods::is_internal_key(name) {
                continue;
            }
            self.map.insert(name.clone(), held.clone());
            self.spread_names.insert(name.clone());
        }
        Ok(())
    }

    /// Read a key, before its value is stored.
    pub(crate) fn key(
        &mut self,
        vm: &mut VirtualMachine,
        key_value: Object,
        key_position: Position,
    ) -> Result<HashKey, MetorexError> {
        // A String key is copied and frozen, so changing the String it was
        // written with afterwards leaves the key alone. The interpreter's own
        // bookkeeping keys are left as they are, and so is a key spelled
        // `:name`, which is how the parser writes the Symbol keys of the
        // keywords a call passes.
        let program_string_key = matches!(key_value.view(), View::String(text)
            if !crate::vm::native_methods::hash_methods::is_internal_key(&text.as_str())
                && !text.as_str().starts_with(':'));
        let key_value = match key_value.view() {
            View::String(text) if program_string_key && !text.is_frozen() => {
                let copy = crate::object::StringValue::clone(text);
                copy.freeze();
                Object::String(Rc::new(copy))
            }
            _ => key_value,
        };
        let mut probe = self.map.clone();
        for (slot, held) in &self.key_objs {
            probe.entry(slot.clone()).or_insert_with(|| held.clone());
        }
        probe.insert(
            "__MX_KEY_OBJECTS__".to_string(),
            Object::Dict(Rc::new(RefCell::new(self.key_objs.clone()))),
        );
        let slot = vm.dict_slot_in(&probe, &key_value, false, key_position)?;
        if !crate::vm::utils::is_primitive_key(&key_value) || program_string_key {
            self.key_objs
                .entry(slot.clone())
                .or_insert_with(|| key_value.clone());
        }
        Ok(HashKey {
            value: key_value,
            slot,
        })
    }

    /// Store the value of a key `key` read.
    pub(crate) fn store(
        &mut self,
        vm: &mut VirtualMachine,
        key: HashKey,
        value: Object,
        written: &WrittenPair,
    ) -> Result<(), MetorexError> {
        if self.map.contains_key(&key.slot)
            && !self.spread_names.contains(&key.slot)
            && written.literal_key
        {
            vm.warn_duplicated_key(&key.value, written)?;
        }
        self.map.insert(key.slot, value);
        Ok(())
    }

    /// Go on building the Hash a chunk of a long literal made.
    pub(crate) fn resume(made: &Object) -> HashLiteral {
        let mut literal = HashLiteral::default();
        if let View::Dict(pairs) = made.view() {
            literal.map = pairs.borrow().clone();
            if let Some(View::Dict(objects)) = literal.map.shift_remove("__MX_KEY_OBJECTS__").view()
            {
                literal.key_objs = objects.borrow().clone();
            }
        }
        literal
    }

    /// The Hash the pairs make.
    pub(crate) fn finish(self) -> Object {
        let mut map = self.map;
        if !self.key_objs.is_empty() {
            map.insert(
                "__MX_KEY_OBJECTS__".to_string(),
                Object::Dict(Rc::new(RefCell::new(self.key_objs))),
            );
        }
        Object::Dict(Rc::new(RefCell::new(map)))
    }
}

pub(crate) fn is_literal_key(key: &Expression) -> bool {
    matches!(
        key,
        Expression::Symbol { .. }
            | Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::NilLiteral { .. }
    )
}
