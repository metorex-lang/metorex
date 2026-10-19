// Where a needle sits inside a string, counted in bytes.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// Where a needle sits in a string, counted in bytes. `byterindex` reads
    /// from the offset backwards, and `byteindex` from the offset forwards.
    pub(crate) fn byte_index_of(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.is_empty() || arguments.len() > 2 {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Range(1, 2),
                arguments.len(),
                position,
            ));
        }
        // `index` and `rindex` count in characters, `byteindex` and
        // `byterindex` in bytes.
        let in_bytes = method_name.starts_with("byte");
        let places = {
            let text = string_value.as_str();
            if in_bytes {
                Places::Bytes(string_value.bytes().len())
            } else if text.is_ascii() {
                Places::Bytes(text.len())
            } else {
                Places::Characters(text.chars().count())
            }
        };
        let total = places.last() as i64;
        let from_end = method_name.ends_with("rindex");
        let pattern = pattern_of(&arguments[0]);
        let searching_pattern = pattern.is_some();
        let refuse_offset = |vm: &mut Self| {
            if searching_pattern {
                vm.globals_mut().set(
                    crate::vm::native_methods::regexp_methods::LAST_MATCH,
                    Object::Nil,
                );
            }
            Object::Nil
        };
        let offset = match arguments.get(1) {
            None => {
                if from_end {
                    total
                } else {
                    0
                }
            }
            Some(held) => {
                let asked = self.integer_argument(method_name, held, position)?;
                let placed = if asked < 0 { asked + total } else { asked };
                if placed < 0 {
                    return Ok(refuse_offset(self));
                }
                if from_end {
                    placed.min(total)
                } else {
                    if placed > total {
                        return Ok(refuse_offset(self));
                    }
                    placed
                }
            }
        };
        if in_bytes {
            crate::vm::native_methods::string_mutation::check_character_boundary(
                &binary_bytes(string_value),
                offset as usize,
                position,
            )?;
        }
        if let Some((pattern, flags)) = pattern {
            self.refuse_mixed_pattern_encoding(string_value, &arguments[0], position)?;
            // The text is read for the search alone, which runs no code of
            // the program's that could change it.
            let text = string_value.as_str();
            let opening = places.byte_at(offset as usize, &text);
            let found =
                self.byte_index_by_pattern(&pattern, &flags, &text, opening, from_end, position)?;
            return Ok(match found.view() {
                View::Int(at) => Object::Int(places.position_of(at as usize, &text)),
                _ => found.clone(),
            });
        }
        let needle = self.string_needle(method_name, &arguments[0], position)?;
        if self
            .compatible_encoding(receiver, &Object::String(Rc::clone(&needle)))
            .is_none()
        {
            let message = format!(
                "incompatible character encodings: {} and {}",
                string_value.encoding_name(),
                needle.encoding_name()
            );
            return Err(crate::vm::errors::simple_exception(
                "Encoding::CompatibilityError",
                &message,
                position,
            ));
        }
        let found = if in_bytes {
            byte_run_index(
                &binary_bytes(string_value),
                &binary_bytes(&needle),
                offset as usize,
                from_end,
            )
            .map(|at| at as i64)
        } else {
            let text = string_value.as_str();
            let opening = places.byte_at(offset as usize, &text);
            byte_run_index(
                text.as_bytes(),
                needle.as_str().as_bytes(),
                opening,
                from_end,
            )
            .map(|at| places.position_of(at, &text))
        };
        Ok(found.map_or(Object::Nil, Object::Int))
    }

    /// The needle a byte search was given: a String, or what `to_str` makes
    /// of the argument. Nothing else names one.
    pub(crate) fn string_needle(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<Rc<crate::object::StringValue>, MetorexError> {
        if let View::String(text) = argument.view() {
            return Ok(Rc::clone(text));
        }
        if let Some(View::String(text)) =
            crate::vm::native_methods::string_subclass_value(argument).view()
        {
            return Ok(text.clone());
        }
        let _ = method_name;
        let named = value_name(self, argument);
        let refuse = |vm: &Self| {
            let _ = vm;
            let message = format!("no implicit conversion of {} into String", named);
            crate::vm::errors::simple_exception("TypeError", &message, position)
        };
        if !self.answers_to(argument, "to_str", position)? {
            return Err(refuse(self));
        }
        match self
            .send_to_object(argument.clone(), "to_str", vec![], position)?
            .view()
        {
            View::String(text) => Ok(text.clone()),
            _ => Err(refuse(self)),
        }
    }

    /// A pattern written in an encoding the subject cannot be read alongside
    /// matches nothing at all, which Ruby reports rather than answering nil.
    fn refuse_mixed_pattern_encoding(
        &mut self,
        string_value: &Rc<crate::object::StringValue>,
        pattern: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let subject = Object::String(Rc::clone(string_value));
        if self.compatible_encoding(&subject, pattern).is_some() {
            return Ok(());
        }
        let named = match self
            .send_to_object(pattern.clone(), "encoding", vec![], position)?
            .view()
        {
            View::Class(held) | View::Module(held) => held.name().to_string(),
            other => other.to_string(),
        };
        let message = format!(
            "incompatible encoding regexp match ({} regexp with {} string)",
            named,
            string_value.encoding_name()
        );
        Err(crate::vm::errors::simple_exception(
            "Encoding::CompatibilityError",
            &message,
            position,
        ))
    }

    /// The byte offset a pattern matches at, reading from `offset` forwards or
    /// backwards. `\G` in the pattern stands for that offset itself.
    fn byte_index_by_pattern(
        &mut self,
        pattern: &str,
        flags: &str,
        text: &str,
        offset: usize,
        from_end: bool,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let found = match crate::vm::native_methods::regexp_methods::previous_match_split(pattern) {
            Some((before, after)) => {
                self.anchored_match_start(&before, &after, flags, text, offset, from_end)
            }
            None => plain_match_start(pattern, flags, text, offset, from_end),
        };
        match found {
            Some(at) => {
                self.regexp_match_data(pattern, flags, text, at, position)?;
                Ok(Object::Int(at as i64))
            }
            None => {
                self.globals_mut().set(
                    crate::vm::native_methods::regexp_methods::LAST_MATCH,
                    Object::Nil,
                );
                Ok(Object::Nil)
            }
        }
    }

    /// Where a pattern carrying `\G` matches: the part before it has to end
    /// at the offset, and the part after it has to start there.
    fn anchored_match_start(
        &mut self,
        before: &str,
        after: &str,
        flags: &str,
        text: &str,
        offset: usize,
        from_end: bool,
    ) -> Option<usize> {
        let tail = crate::vm::native_methods::regexp_methods::compile(after, flags)?;
        let held = tail.find_at(text, offset)?;
        if held.start() != offset {
            return None;
        }
        let head = crate::vm::native_methods::regexp_methods::compile(
            &format!(r"(?:{})\z", before),
            flags,
        )?;
        let opening = &text[..offset];
        let mut starts: Vec<usize> = Vec::new();
        for at in 0..=offset {
            if !opening.is_char_boundary(at) {
                continue;
            }
            if head
                .find_at(opening, at)
                .is_some_and(|found| found.start() == at)
            {
                starts.push(at);
            }
        }
        if from_end {
            starts.pop()
        } else {
            starts.first().copied()
        }
    }
}

/// The pattern and flags a value stands for, reading a Regexp subclass
/// through the pattern it keeps.
pub(crate) fn pattern_of(value: &Object) -> Option<(String, String)> {
    match value.view() {
        View::Regex(pattern, flags) => Some((pattern.to_string(), flags.to_string())),
        _ => match crate::vm::native_methods::regexp_subclass_value(value).view() {
            Some(View::Regex(pattern, flags)) => Some((pattern.to_string(), flags.to_string())),
            _ => None,
        },
    }
}

/// Where a pattern with no `\G` matches, reading from the offset forwards or
/// backwards.
fn plain_match_start(
    pattern: &str,
    flags: &str,
    text: &str,
    offset: usize,
    from_end: bool,
) -> Option<usize> {
    let compiled = crate::vm::native_methods::regexp_methods::compile(pattern, flags)?;
    if !from_end {
        return compiled.find_at(text, offset).map(|found| found.start());
    }
    let mut at = offset;
    loop {
        if text.is_char_boundary(at)
            && compiled
                .find_at(text, at)
                .is_some_and(|found| found.start() == at)
        {
            return Some(at);
        }
        if at == 0 {
            return None;
        }
        at -= 1;
    }
}

/// Where one run of bytes sits inside another, reading from the offset
/// forwards or backwards.
fn byte_run_index(bytes: &[u8], wanted: &[u8], offset: usize, from_end: bool) -> Option<usize> {
    let last = bytes.len().checked_sub(wanted.len())?;
    if from_end {
        let mut at = offset.min(last);
        loop {
            if bytes[at..at + wanted.len()] == *wanted {
                return Some(at);
            }
            if at == 0 {
                return None;
            }
            at -= 1;
        }
    }
    (offset..=last).find(|at| bytes[*at..*at + wanted.len()] == *wanted)
}

/// The name Ruby gives a value when it reports a conversion it could not
/// make: `nil`, `true`, and `false` name themselves, and everything else
/// names its class.
fn value_name(vm: &VirtualMachine, held: &Object) -> String {
    match held.view() {
        View::Nil => "nil".to_string(),
        View::Bool(true) => "true".to_string(),
        View::Bool(false) => "false".to_string(),
        _ => vm.builtins().class_of(held).name().to_string(),
    }
}

/// The position a byte offset stands at: the offset itself when positions are
/// counted in bytes, and the number of characters before it otherwise.
/// What the positions a search counts in are, each numbered from 0 with
/// one more for the end of the string.
enum Places {
    /// The bytes of a string this many bytes long, which is what a search
    /// counting in bytes, or in characters of ASCII text, counts.
    Bytes(usize),
    /// The characters of a string this many characters long.
    Characters(usize),
}

impl Places {
    /// The number of the last position, the end of the string.
    fn last(&self) -> usize {
        match self {
            Places::Bytes(count) | Places::Characters(count) => *count,
        }
    }

    /// Where position `index` of `text` starts, in bytes.
    fn byte_at(&self, index: usize, text: &str) -> usize {
        match self {
            Places::Bytes(_) => index,
            Places::Characters(_) => text
                .char_indices()
                .nth(index)
                .map_or(text.len(), |(at, _)| at),
        }
    }

    /// The position the byte `at` of `text` stands at.
    fn position_of(&self, at: usize, text: &str) -> i64 {
        match self {
            Places::Bytes(_) => at as i64,
            Places::Characters(_) => text[..at].chars().count() as i64,
        }
    }
}
