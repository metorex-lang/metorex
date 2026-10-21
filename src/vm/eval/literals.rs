// The literals and names the bytecode runner reads.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Object, OptionView, View};
use crate::vm::core::VirtualMachine;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    /// A new String of a literal's text.
    pub(crate) fn evaluate_string_literal(&mut self, value: &str, position: Position) -> Object {
        // A literal is written in the encoding its source is written
        // in, which a magic comment at the top of the file names.
        // A literal written in bytes carries its own encoding, so one
        // reaching here with anything outside ASCII was spelled with a
        // `\u` escape and stands for those codepoints.
        let made = match self.source_literal_encoding() {
            Some(named) if value.is_ascii() => {
                crate::object::StringValue::with_encoding(value.to_string(), named)
            }
            // A file written in an encoding of its own spells a
            // character with bytes of that encoding, so the literal
            // carries those bytes rather than the UTF-8 the source was
            // read into.
            Some(named)
                if let Some(bytes) =
                    crate::vm::native_methods::string_methods::spelled_bytes(value, &named)
                        .or_else(|| crate::file_loader::escaped_source_bytes(value)) =>
            {
                let held = crate::object::StringValue::with_encoding(
                    crate::vm::native_methods::string_methods::bytes_as_text(&bytes),
                    named,
                );
                held.mark_bytes();
                held
            }
            _ => crate::object::StringValue::new(value.to_string()),
        };
        // Ruby 3.4 hands a literal back with notice that a later
        // release will freeze it, so the first change made to it says
        // so. A source that asked for frozen literals produces a
        // frozen one instead and never reaches here.
        // A run told to report where a literal was written names the
        // place instead of pointing at the flag that would name it.
        match self.literal_birthplace(position) {
            Some(written_at) => {
                made.set_created_at(written_at);
                made.chill("warning: literal string will be frozen in the future".to_string());
            }
            None => made.chill(
                "warning: literal string will be frozen in the future (run with \
--debug-frozen-string-literal for more information)"
                    .to_string(),
            ),
        }
        let made = Object::String(std::rc::Rc::new(made));
        self.record_allocation(&made, position);
        made
    }

    /// The Symbol a literal names.
    pub(crate) fn evaluate_symbol_literal(&mut self, value: &str) -> Object {
        // A symbol is named in the encoding the source naming it is
        // written in, which is what `Symbol#encoding` reports for one
        // whose name is not all ASCII.
        let named = self
            .current_source_encoding
            .clone()
            .map(|held| crate::vm::native_methods::string_methods::canonical_encoding_name(&held))
            .unwrap_or_else(|| crate::object::string_value::DEFAULT_ENCODING.to_string());
        // A source written in bytes names its symbols in bytes, so
        // each character of the name stands for one of them.
        // A source written in an encoding of its own names its
        // symbols in the bytes that encoding spells them with.
        let plain = named == crate::object::string_value::DEFAULT_ENCODING;
        if plain || value.is_ascii() {
            let made = crate::object::StringValue::with_encoding(value.to_string(), named);
            return Object::Symbol(std::rc::Rc::new(made));
        }
        // A source read as bytes names its symbols in the bytes the
        // file itself holds, which is the text as it was written.
        let spelled = if matches!(named.as_str(), "ASCII-8BIT" | "BINARY") {
            Some(value.as_bytes().to_vec())
        } else {
            crate::vm::native_methods::string_methods::spelled_bytes(value, &named)
        };
        let Some(bytes) = spelled else {
            let made = crate::object::StringValue::with_encoding(value.to_string(), named);
            return Object::Symbol(std::rc::Rc::new(made));
        };
        let made = crate::object::StringValue::with_encoding(
            crate::vm::native_methods::string_methods::bytes_as_text(&bytes),
            named,
        );
        made.mark_bytes();
        Object::Symbol(std::rc::Rc::new(made))
    }

    /// The value of an identifier: a local, a constant, or a call on self.
    pub(crate) fn read_identifier(
        &mut self,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // A name that is no local is a call the program made.
        let pending =
            std::mem::replace(&mut self.program_call_pending, !self.tracepoints.is_empty());
        let value = self.eval_identifier(name, position);
        self.program_call_pending = pending;
        let value = value?;
        if name.starts_with(|first: char| first.is_ascii_uppercase()) && self.in_a_non_main_ractor()
        {
            let (owner, found_where_written) = self.constant_owner_name(name);
            self.refuse_unshareable_constant(
                &format!("{owner}::{name}"),
                found_where_written,
                &value,
                position,
            )?;
        }
        Ok(value)
    }

    /// The Regexp a literal makes.
    pub(crate) fn evaluate_regex_literal(
        &mut self,
        pattern: &str,
        flags: &str,
        position: Position,
    ) -> Object {
        // A repetition written on a repetition reads as one, which
        // Ruby says so about where the pattern is read.
        if pattern.contains("}+")
            && let Ok(read) =
                crate::vm::native_methods::regexp_methods::read_pattern(pattern, flags)
        {
            for warning in &read.warnings {
                self.emit_warning_to_stderr(&format!("warning: {warning}"), position);
            }
        }
        Object::Regex(Rc::new(pattern.to_string()), Rc::new(flags.to_string()))
    }

    /// `__FILE__`.
    pub(crate) fn magic_file(&self) -> Object {
        // The file the code was written in, which is not the file
        // being run when a required file's method or block is what is
        // executing.
        let path = self
            .current_source_file
            .clone()
            .or_else(|| {
                self.reported_current_file()
                    .map(|path| path.display().to_string())
            })
            .unwrap_or_else(|| "(eval)".to_string());
        Object::string(path)
    }

    /// `__dir__`.
    pub(crate) fn magic_dir(&self) -> Object {
        // `__dir__` is nil where there is no file behind the code, and
        // otherwise the directory holding it, which is "." for a bare
        // filename the way `File.dirname` reports it.
        // The directory is the one holding the file the code was
        // written in, which is what `__FILE__` names, rather than the
        // file being run when a required file's block is executing.
        let written_in = self
            .current_source_file
            .clone()
            .map(std::path::PathBuf::from)
            .or_else(|| self.get_current_file().cloned());
        let Some(file) = written_in else {
            return Object::Nil;
        };
        // Code handed to `eval` with no filename of its own is named
        // for the place the eval was written, which is not a file and
        // so holds no directory.
        if file
            .to_str()
            .is_some_and(|named| named.starts_with(crate::vm::EVAL_FILE_PREFIX))
        {
            return Object::Nil;
        }
        let file = file.as_path();
        // A relative path expands against the working directory, so
        // running `metorex script.rb` still names the real directory.
        let directory = match std::fs::canonicalize(file) {
            Ok(resolved) => match resolved.parent() {
                Some(parent) => parent.display().to_string(),
                None => ".".to_string(),
            },
            Err(_) => match file.parent().map(|d| d.display().to_string()) {
                Some(directory) if !directory.is_empty() => directory,
                _ => ".".to_string(),
            },
        };
        Object::string(directory)
    }

    /// `::Name`, which reads the top level directly, skipping the lexical
    /// chain and any class-local constant of the same name.
    pub(crate) fn read_top_level_constant(
        &mut self,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // `::Name` writes Object out as the scope, which a private
        // constant refuses.
        if let Some(View::Class(object_class)) = self.globals().get("Object").view()
            && object_class.is_private_constant(name)
        {
            return self.private_constant_refused(object_class, object_class, name, position);
        }
        if let Some(value) = self
            .globals()
            .get(name)
            .or_else(|| self.object_constant(name))
        {
            self.refuse_unshareable_constant(&format!("Object::{name}"), false, &value, position)?;
            return Ok(value);
        }
        // A name registered as an autoload at the top level is loaded
        // the first time it is written out with `::`.
        if let Some(View::Class(object_class)) = self.globals().get("Object").view()
            && let Some(value) = self.try_autoload_constant(object_class, name)?
        {
            return Ok(value);
        }
        let message = format!("uninitialized constant {}", name);
        Err(MetorexError::UncaughtException {
            exception: self.constant_name_error(&message, name),
            location: position_to_location(position),
            message,
        })
    }

    /// `namespace::Name`, with the namespace evaluated.
    pub(crate) fn read_scope_resolution(
        &mut self,
        ns_value: Object,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let value = self.read_scoped_constant(ns_value.clone(), name, &position)?;
        if self.in_a_non_main_ractor() {
            let owner = match ns_value.view() {
                View::Class(held) | View::Module(held) => held.ruby_name(),
                other => format!("{other}"),
            };
            self.refuse_unshareable_constant(&format!("{owner}::{name}"), false, &value, position)?;
        }
        Ok(value)
    }

    /// The Range a literal makes of its evaluated ends. A range written with
    /// literal ends is one object, however many times the line it sits on is
    /// run, so the mark it carries is kept against where it was written.
    pub(crate) fn build_range(
        &mut self,
        start_value: Object,
        end_value: Object,
        exclusive: bool,
        literal_ends: bool,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.check_range_ends(&start_value, &end_value, position)?;
        let written_at = literal_ends.then(|| {
            (
                self.current_source_file.clone().unwrap_or_default(),
                position.line,
                position.column,
            )
        });
        // A range written with literal ends is built once, so the Strings at
        // its ends are frozen with it.
        if written_at.is_some() {
            for held in [&start_value, &end_value] {
                if let View::String(text) = held.view() {
                    text.freeze();
                }
            }
        }
        let mark = match written_at {
            Some(place) => self
                .written_ranges
                .entry(place)
                .or_insert_with(|| Rc::new(()))
                .clone(),
            None => Rc::new(()),
        };
        Ok(Object::Range(
            Box::new(start_value),
            Box::new(end_value),
            exclusive,
            mark,
        ))
    }

    /// `collection[key]`, with both evaluated.
    pub(crate) fn index_value(
        &mut self,
        collection: Object,
        key: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Block/Lambda [] call syntax: proc[args]
        if let View::Block(block) = collection.view() {
            return block.call(self, vec![key], position);
        }
        // `native_fn [args]` — treat as a call with the bracketed array as a single argument.
        // This matches Ruby's `private [:foo, :bar]` which passes an Array to `private`.
        if let View::NativeFunction(name) = collection.view() {
            return self.call_native_function(&name.clone(), vec![key], position);
        }
        // A `[]` the program wrote on Array or Hash answers in place of the
        // direct read, except in the core library's own code, which reads
        // the way MRI's C code does.
        let read_directly = position.prelude
            || match collection.view() {
                View::Array(_) => self.array_read_in_force(&collection),
                View::Dict(_) => self.hash_read_in_force(&collection),
                _ => true,
            };
        if !read_directly {
            return self.call_with_block(
                collection,
                "[]",
                vec![key],
                None,
                crate::vm::method_lookup::WrittenReceiver::Other,
                position,
            );
        }
        // Check for user-defined [] method on instances
        if let View::Instance(instance_rc) = collection.view() {
            let class = Rc::clone(&instance_rc.borrow().class);
            if let Some(method) = class.find_method("[]") {
                return self.invoke_method(class, method, collection.clone(), vec![key], position);
            }
            // A Thread keeps its locals under `t[:k]`, which the
            // Thread method table reads. Going through it here keeps
            // one reading of the key for the subscript and the call.
            if class.name() == "Thread" {
                return self
                    .call_thread_method(&collection, "[]", std::slice::from_ref(&key), position)?
                    .ok_or_else(|| {
                        MetorexError::runtime_error(
                            "Thread lookup answered nothing",
                            position_to_location(position),
                        )
                    });
            }
        }
        // A number reads its own bits by subscript, where a pair of
        // arguments arrives as the Array the subscript gathered.
        if matches!(collection.view(), View::Int(_) | View::BigInt(_)) {
            let spread = match key.view() {
                View::Array(gathered) => gathered.borrow().clone(),
                _ => vec![key.clone()],
            };
            return self.send_to_object(collection, "[]", spread, position);
        }
        self.evaluate_index_operation(collection, key, position)
    }

    /// `value => pattern`, which refuses a value the pattern does not cover,
    /// and `value in pattern`, which answers whether it does.
    pub(crate) fn test_pattern(
        &mut self,
        held: Object,
        pattern: &crate::ast::MatchPattern,
        refuses: bool,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut bindings = std::collections::HashMap::new();
        self.pattern_failure = None;
        let matched = self.match_pattern(pattern, &held, &mut bindings, position)?;
        if matched {
            self.apply_pattern_bindings(&bindings);
        } else if refuses {
            return Err(self.detailed_no_matching_pattern(&held, position));
        }
        Ok(if refuses {
            Object::Nil
        } else {
            Object::Bool(matched)
        })
    }
}
