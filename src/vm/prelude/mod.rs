// Core library pieces defined in Ruby rather than in Rust.
//
// A method written here is a real user-defined method, so it can be aliased,
// redefined, mocked, and introspected the way MRI's own Ruby-level core
// methods can. Kernel#warn relies on that: the specs alias `Warning.warn`
// away and put their own back.
//
// The Ruby source is split across the modules chunks.rs lists. Each holds
// one `SOURCE` string, and they are concatenated in order, so a chunk
// boundary falls between two definitions. A class whose body runs long is
// carried across more than one module, so a boundary may sit inside a class
// body. The build script parses the source, and the binary holds what it
// parsed.

use crate::object::OptionView;
use crate::vm::core::VirtualMachine;

/// What names the core library's own Ruby source. Ruby reports a method
/// written there as coming from `<internal:...>` rather than from a file.
pub(crate) const PRELUDE_FILE: &str = "<internal:prelude>";

impl VirtualMachine {
    /// Build an `Enumerator` over `method_name` sent to `receiver`, which is
    /// what a method that yields answers when called without a block.
    pub(crate) fn build_enumerator(
        &mut self,
        receiver: crate::object::Object,
        method_name: &str,
        arguments: Vec<crate::object::Object>,
        size: Option<i64>,
        position: crate::lexer::Position,
    ) -> Result<crate::object::Object, crate::error::MetorexError> {
        use crate::object::Object;
        let Some(enumerator_class) = self.globals().get("Enumerator") else {
            let message = "uninitialized constant Enumerator".to_string();
            return Err(crate::error::MetorexError::UncaughtException {
                exception: Object::exception("NameError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        };
        let arguments = vec![
            receiver,
            Object::symbol(method_name.to_string()),
            Object::Array(std::rc::Rc::new(std::cell::RefCell::new(arguments))),
            match size {
                Some(size) => Object::Int(size),
                None => Object::Nil,
            },
        ];
        self.send_to_object(enumerator_class, "over", arguments, position)
    }

    /// A walk whose count is named by an object rather than a plain Integer,
    /// which is how an endless walk reports a size of Infinity.
    pub(crate) fn build_enumerator_of_size(
        &mut self,
        receiver: crate::object::Object,
        method_name: &str,
        arguments: Vec<crate::object::Object>,
        size: crate::object::Object,
        position: crate::lexer::Position,
    ) -> Result<crate::object::Object, crate::error::MetorexError> {
        use crate::object::Object;
        let Some(enumerator_class) = self.globals().get("Enumerator") else {
            let message = "uninitialized constant Enumerator".to_string();
            return Err(crate::error::MetorexError::UncaughtException {
                exception: Object::exception("NameError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        };
        let arguments = vec![
            receiver,
            Object::symbol(method_name.to_string()),
            Object::Array(std::rc::Rc::new(std::cell::RefCell::new(arguments))),
            size,
        ];
        self.send_to_object(enumerator_class, "over", arguments, position)
    }

    /// Evaluate the Ruby-level core library. A runtime failure here is a
    /// defect in the library itself, so it panics rather than leaving a
    /// half-built VM behind.
    pub(crate) fn load_prelude(&mut self) {
        let statements = crate::vm::parsed_sources::prelude();
        // The core library is no file of the program's, so a method written
        // here names itself the way Ruby names its own Ruby-level core.
        let held_file = self
            .current_file
            .replace(std::path::PathBuf::from(PRELUDE_FILE));
        let held_source = self.current_source_file.replace(PRELUDE_FILE.to_string());
        self.execute_program(&statements)
            .unwrap_or_else(|error| panic!("prelude failed to run: {}", error));
        self.current_file = held_file;
        self.current_source_file = held_source;
        self.give_main_its_private_methods();
        self.take_ordering_from_complex();
    }

    /// A Complex names no point on the number line, so it is neither ordered
    /// against another number nor rounded. The methods that would do either
    /// come to it natively, so they are undefined here rather than by an
    /// `undef_method`, which looks for them in a method table.
    fn take_ordering_from_complex(&mut self) {
        let complex = self.globals().get("Complex");
        let Some(crate::object::View::Class(complex)) = complex.view() else {
            return;
        };
        for name in [
            "%",
            "<",
            "<=",
            ">",
            ">=",
            "between?",
            "clamp",
            "div",
            "divmod",
            "floor",
            "ceil",
            "modulo",
            "remainder",
            "round",
            "step",
            "truncate",
            "i",
        ] {
            complex.define_method(
                name,
                std::rc::Rc::new(crate::object::Method::undefined(name.to_string())),
            );
        }
    }

    /// The private methods the singleton class of `main` defines natively,
    /// each running the way its bare form written at the top level runs.
    fn give_main_its_private_methods(&mut self) {
        let main = self.globals().get("__main__");
        let Some(crate::object::View::Instance(main)) = main.view() else {
            return;
        };
        let Some(singleton) = main.borrow().singleton_class() else {
            return;
        };
        for name in [
            "define_method",
            "include",
            "private",
            "public",
            "ruby2_keywords",
            "using",
        ] {
            let held = crate::parser::ANONYMOUS_SPLAT.to_string();
            let mut stub = crate::object::Method::new(name.to_string(), vec![held.clone()], vec![]);
            stub.signature.variadic_param = Some((0, held));
            stub.provenance.native_alias = Some(name.to_string());
            singleton.define_method(name, std::rc::Rc::new(stub));
            singleton.set_method_private(name);
        }
    }
}
