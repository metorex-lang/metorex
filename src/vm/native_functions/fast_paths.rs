// Rex's native fast paths, and the guards they run behind.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// `__fast_paths__(on)`: whether the native fast paths run, which Rex
    /// sets from `config.fast_paths`.
    pub(crate) fn set_fast_paths(&mut self, arguments: &[Object]) -> Result<Object, MetorexError> {
        let on = arguments.first().is_some_and(Object::is_truthy);
        self.fast_paths_enabled = on;
        Ok(Object::Bool(on))
    }

    /// `__guard_fast_path__(owner, name)`: records `owner`'s method `name` as
    /// one a fast path stands in for, so the fast path runs only while the
    /// method a receiver answers with is this one.
    pub(crate) fn guard_fast_path(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (Some(View::Class(owner) | View::Module(owner)), Some(name)) =
            (arguments.first().view(), arguments.get(1))
        else {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                "__guard_fast_path__ takes a module and a method name",
                position,
            ));
        };
        let name = self.coerce_method_name(name, "__guard_fast_path__", position)?;
        let Some(original) = owner.find_method(&name) else {
            return Err(crate::vm::errors::simple_exception(
                "NameError",
                &format!("undefined method '{name}' for {}", owner.ruby_name()),
                position,
            ));
        };
        self.fast_path_originals
            .entry(name)
            .or_default()
            .push(original);
        Ok(Object::Nil)
    }

    /// Whether `class` answers `name` with a method a fast path recorded,
    /// checked again only after the method state moved.
    fn answers_as_recorded(&mut self, class: &Rc<crate::class::Class>, name: &str) -> bool {
        let state = crate::class::method_state();
        let key = (Rc::as_ptr(class) as usize, name.to_string());
        if let Some((checked_under, held)) = self.fast_path_checks.get(&key)
            && *checked_under == state
        {
            return *held;
        }
        let held = match (class.find_method(name), self.fast_path_originals.get(name)) {
            (Some(answered), Some(originals)) => originals
                .iter()
                .any(|original| Rc::ptr_eq(original, &answered)),
            _ => false,
        };
        self.fast_path_checks.insert(key, (state, held));
        held
    }

    /// `__let_value__(name) { ... }`: what the `let` named `name` holds for
    /// the running example. While the fast paths run and `__memoized` and
    /// `fetch_or_store` are Rex's own, a value already stored is read from
    /// the example's memo hash directly. Anything else goes through
    /// `__memoized.fetch_or_store(name) { ... }`, as `let` does in RSpec.
    pub(crate) fn let_value(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let block = self.pending_block.take();
        let name = arguments.first().cloned().unwrap_or(Object::Nil);
        let receiver = self.environment().get("self").unwrap_or(Object::Nil);
        if self.fast_paths_enabled
            && let Some(stored) = self.stored_let_value(&receiver, &name)
        {
            return Ok(stored);
        }
        let memoized = self.send_to_object(receiver, "__memoized", Vec::new(), position)?;
        self.pending_block = block;
        self.send_to_object(memoized, "fetch_or_store", vec![name], position)
    }

    /// The value the example `receiver` stored for the `let` `name`, when
    /// its memo is Rex's own and holds one.
    fn stored_let_value(&mut self, receiver: &Object, name: &Object) -> Option<Object> {
        let View::Instance(example) = receiver.view() else {
            return None;
        };
        let example_class = Rc::clone(&example.borrow().class);
        if !self.answers_as_recorded(&example_class, "__memoized") {
            return None;
        }
        let memo = example.borrow().get_var("__memoized").cloned()?;
        let View::Instance(memo) = memo.view() else {
            return None;
        };
        let memo_class = Rc::clone(&memo.borrow().class);
        if !self.answers_as_recorded(&memo_class, "fetch_or_store") {
            return None;
        }
        let viewed = memo.borrow().get_var("memoized").cloned();
        let Some(View::Dict(slots)) = viewed.view() else {
            return None;
        };
        let slot = crate::vm::utils::object_to_dict_key(name)?;
        slots.borrow().get(slot.as_str()).cloned()
    }
}

/// The methods of `ExampleHash` the native metadata populator stands in for.
const POPULATE_STEPS: [&str; 12] = [
    "populate",
    "ensure_valid_user_keys",
    "populate_location_attributes",
    "build_description_from",
    "description_separator",
    "build_scoped_id_for",
    "full_description",
    "described_class",
    "metadata",
    "user_metadata",
    "block",
    "description_args",
];

impl VirtualMachine {
    /// `__populate_example_metadata__(populator, reserved_keys, metadata_module)`:
    /// fills an example's metadata as `ExampleHash#populate` does, and
    /// answers true. Answers false, having written nothing, when the fast
    /// paths are off, a step of `populate` was redefined, or the example
    /// asks for something only the Ruby steps handle: a `:caller`, a
    /// reserved key, no block, or a description that is not a String or
    /// Symbol.
    pub(crate) fn populate_example_metadata(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (Some(View::Instance(populator)), Some(View::Array(reserved)), Some(metadata_module)) = (
            arguments.first().view(),
            arguments.get(1).view(),
            arguments.get(2),
        ) else {
            return Ok(Object::Bool(false));
        };
        if !self.fast_paths_enabled {
            return Ok(Object::Bool(false));
        }
        let populator_class = Rc::clone(&populator.borrow().class);
        for step in POPULATE_STEPS {
            if !self.answers_as_recorded(&populator_class, step) {
                return Ok(Object::Bool(false));
            }
        }
        let held = |name: &str| populator.borrow().get_var(name).cloned();
        let (metadata, user_metadata) = (held("metadata"), held("user_metadata"));
        let (Some(View::Dict(metadata)), Some(View::Dict(user_metadata))) =
            (metadata.view(), user_metadata.view())
        else {
            return Ok(Object::Bool(false));
        };
        let (Some(index_provider), Some(description_args), Some(block)) = (
            held("index_provider"),
            held("description_args"),
            held("block"),
        ) else {
            return Ok(Object::Bool(false));
        };
        if !matches!(description_args.view(), View::Array(_)) {
            return Ok(Object::Bool(false));
        }
        let slot = |name: &str| format!(":{name}");
        let user_slots: Vec<crate::object::Slot> = user_metadata.borrow().keys().cloned().collect();
        let reserved_slots: Vec<crate::object::Slot> = reserved
            .borrow()
            .iter()
            .filter_map(crate::vm::utils::object_to_dict_key)
            .collect();
        if user_slots
            .iter()
            .any(|held| *held == slot("caller") || reserved_slots.contains(held))
        {
            return Ok(Object::Bool(false));
        }
        let View::Array(arguments_held) = description_args.view() else {
            return Ok(Object::Bool(false));
        };
        let first_argument = arguments_held.borrow().first().cloned();
        let description = match first_argument.as_ref().view() {
            None => Object::string(String::new()),
            Some(text @ View::String(_)) => text.to_object(),
            Some(View::Symbol(name)) => Object::string(name.to_string()),
            Some(_) => return Ok(Object::Bool(false)),
        };
        let group = match metadata.borrow().get(slot("example_group").as_str()).view() {
            Some(View::Dict(group)) => Rc::clone(group),
            _ => return Ok(Object::Bool(false)),
        };
        let (group_description, described_class) = {
            let group = group.borrow();
            match (
                group.get(slot("full_description").as_str()).view(),
                group.get(slot("described_class").as_str()),
            ) {
                (Some(View::String(text)), Some(described)) => {
                    (text.as_str().to_string(), described.clone())
                }
                _ => return Ok(Object::Bool(false)),
            }
        };
        let full_description = if group_description.is_empty() {
            description.clone()
        } else {
            let View::String(text) = description.view() else {
                return Ok(Object::Bool(false));
            };
            Object::string(format!("{group_description} {}", text.as_str()))
        };

        // An example written without a block finds its place from the
        // caller, which the Ruby steps read.
        if !matches!(block.view(), View::Block(_)) {
            return Ok(Object::Bool(false));
        }
        let source = self.send_to_object(block.clone(), "source_location", Vec::new(), position)?;
        let (file_path, line_number) = match source.view() {
            View::Array(pair) => {
                let pair = pair.borrow();
                match (pair.first().view(), pair.get(1).view()) {
                    (Some(file @ View::String(_)), Some(View::Int(line))) => {
                        (file.to_object(), line)
                    }
                    _ => return Ok(Object::Bool(false)),
                }
            }
            _ => return Ok(Object::Bool(false)),
        };
        let relative = self.send_to_object(
            metadata_module.clone(),
            "relative_path",
            vec![file_path.clone()],
            position,
        )?;
        let file_class = self.globals().get("File").unwrap_or(Object::Nil);
        let absolute =
            self.send_to_object(file_class, "expand_path", vec![relative.clone()], position)?;
        let index =
            self.send_to_object(index_provider, "call", vec![absolute.clone()], position)?;
        let index = match index.view() {
            View::Int(number) => number.to_string(),
            _ => return Ok(Object::Bool(false)),
        };
        let View::String(relative_text) = relative.view() else {
            return Ok(Object::Bool(false));
        };
        let location = Object::string(format!("{}:{line_number}", relative_text.as_str()));
        let scoped_id = match metadata.borrow().get(slot("scoped_id").as_str()).view() {
            Some(View::String(parent)) => Object::string(format!("{}:{index}", parent.as_str())),
            Some(_) => return Ok(Object::Bool(false)),
            None => Object::string(index),
        };
        let rerun_file_path = match metadata.borrow().get(slot("rerun_file_path").as_str()) {
            Some(existing) if existing.is_truthy() => existing.clone(),
            _ => relative.clone(),
        };

        let entries = [
            ("block", block),
            ("description_args", description_args.clone()),
            ("description", description),
            ("full_description", full_description),
            ("described_class", described_class),
            ("file_path", relative),
            ("line_number", Object::Int(line_number)),
            ("location", location),
            ("absolute_file_path", absolute),
            ("rerun_file_path", rerun_file_path),
            ("scoped_id", scoped_id),
        ];
        for (name, value) in entries {
            self.hash_store(metadata, &Object::symbol(name.to_string()), value, position)?;
        }
        for (key, value) in self.hash_pairs(user_metadata) {
            self.hash_store(metadata, &key, value, position)?;
        }
        // `Example#initialize` stores `:last_run_status` next, which the
        // entries are left room for. They are moved to a block of that size,
        // since shrinking the one they grew into leaves it larger than asked
        // for.
        let mut entries = metadata.borrow_mut();
        let mut rebuilt = crate::object::HashEntries::with_capacity_and_hasher(
            entries.len() + 1,
            Default::default(),
        );
        rebuilt.extend(entries.drain(..));
        *entries = rebuilt;
        Ok(Object::Bool(true))
    }
}

impl VirtualMachine {
    /// `__new_example_group__(group, inspect_output, memo_class,
    /// reentrant_mutex_class, example_group)`: an instance of `group` for
    /// one example, built as `group.new(inspect_output)` builds it, with
    /// the same instance variables set in the same order. Answers nil,
    /// having built nothing, when the fast paths are off or the
    /// initializers that build it are not Rex's own: one a group or a
    /// module it includes defines, a `new` of its own, or a redefined one.
    pub(crate) fn new_example_group(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (
            Some(View::Class(group)),
            Some(inspect_output),
            Some(View::Class(memo_class)),
            Some(reentrant_mutex_class),
            Some(View::Class(example_group)),
        ) = (
            arguments.first().view(),
            arguments.get(1),
            arguments.get(2).view(),
            arguments.get(3),
            arguments.get(4).view(),
        )
        else {
            return Ok(Object::Nil);
        };
        // A thread-safe memo holds a reentrant mutex, which the class given
        // for one builds.
        let reentrant_mutex_class = match reentrant_mutex_class.view() {
            View::Class(class) => Some(class),
            _ => None,
        };
        if !self.fast_paths_enabled
            || !self.builds_as_rex_does(group, memo_class, reentrant_mutex_class, example_group)
        {
            return Ok(Object::Nil);
        }
        let instance = self.allocate_instance(group, position)?;
        let memo = self.allocate_instance(memo_class, position)?;
        let (View::Instance(held), View::Instance(memo_held)) = (instance.view(), memo.view())
        else {
            return Ok(Object::Nil);
        };
        let inspect_output = if inspect_output.is_truthy() {
            inspect_output.clone()
        } else {
            Object::string("(no description provided)")
        };
        held.borrow_mut()
            .set_program_var("__inspect_output".to_string(), inspect_output);
        memo_held
            .borrow_mut()
            .set_var("memoized".to_string(), Object::empty_dict());
        if let Some(reentrant_mutex_class) = reentrant_mutex_class {
            let mutex = self.allocate_instance(reentrant_mutex_class, position)?;
            if let View::Instance(mutex_held) = mutex.view() {
                let lock = self.globals().get("Mutex").unwrap_or(Object::Nil);
                let lock = self.send_to_object(lock, "new", Vec::new(), position)?;
                let mut mutex_held = mutex_held.borrow_mut();
                mutex_held.set_var("owner".to_string(), Object::Nil);
                mutex_held.set_var("count".to_string(), Object::Int(0));
                mutex_held.set_var("mutex".to_string(), lock);
            }
            memo_held.borrow_mut().set_var("mutex".to_string(), mutex);
        }
        held.borrow_mut()
            .set_program_var("__memoized".to_string(), memo);
        Ok(instance)
    }

    /// Whether `group.new` runs Rex's own initializers and nothing else,
    /// checked again only after the method state moved.
    fn builds_as_rex_does(
        &mut self,
        group: &Rc<crate::class::Class>,
        memo_class: &Rc<crate::class::Class>,
        reentrant_mutex_class: Option<&Rc<crate::class::Class>>,
        example_group: &Rc<crate::class::Class>,
    ) -> bool {
        let state = crate::class::method_state();
        let key = (
            Rc::as_ptr(group) as usize,
            format!(
                "__new_example_group__ {:p} {}",
                Rc::as_ptr(memo_class),
                reentrant_mutex_class.is_some()
            ),
        );
        if let Some((checked_under, held)) = self.fast_path_checks.get(&key)
            && *checked_under == state
        {
            return *held;
        }
        let recorded = |vm: &Self, method: &Rc<crate::object::Method>| {
            vm.fast_path_originals
                .get("initialize")
                .is_some_and(|originals| {
                    originals
                        .iter()
                        .any(|original| Rc::ptr_eq(original, method))
                })
        };
        // A `new` or `allocate` of a group's own, or of a group around it.
        let mut cursor = Some(Rc::clone(group));
        let mut own_constructor = false;
        while let Some(current) = cursor {
            let on_singleton = current
                .singleton_class_slot()
                .as_ref()
                .is_some_and(|singleton| {
                    singleton.find_own_method("new").is_some()
                        || singleton.find_own_method("allocate").is_some()
                });
            if on_singleton
                || current.find_own_method("__class__new").is_some()
                || current.find_own_method("__class__allocate").is_some()
            {
                own_constructor = true;
                break;
            }
            if Rc::ptr_eq(&current, example_group) {
                break;
            }
            cursor = current.superclass();
        }
        // An `initialize` of a module ExampleGroup includes runs between
        // ExampleGroup's and MemoizedHelpers'.
        let module_initializer = example_group.transitive_mixins().iter().any(|module| {
            module
                .find_own_method("initialize")
                .is_some_and(|method| !recorded(self, &method))
        });
        let object_initializer = example_group
            .superclass()
            .and_then(|object| object.find_method("initialize"))
            .is_some();
        let held = !own_constructor
            && !module_initializer
            && !object_initializer
            && self.answers_as_recorded(group, "initialize")
            && self.answers_as_recorded(group, "__init_memoized")
            && self.answers_as_recorded(memo_class, "initialize")
            && reentrant_mutex_class
                .is_none_or(|class| self.answers_as_recorded(class, "initialize"));
        self.fast_path_checks.insert(key, (state, held));
        held
    }
}

/// The method a class or module answers itself, in the order a call to it
/// looks: its singleton class's own, a `def self.name`, what its singleton
/// class inherits, and what it extended.
fn class_level_method(
    owner: &Rc<crate::class::Class>,
    name: &str,
) -> Option<Rc<crate::object::Method>> {
    owner
        .singleton_class_slot()
        .as_ref()
        .and_then(|singleton| singleton.find_own_method(name))
        .or_else(|| crate::vm::method_lookup::module_own_method(owner, name))
        .or_else(|| {
            owner
                .singleton_class_slot()
                .as_ref()
                .and_then(|singleton| singleton.find_method(name))
        })
        .or_else(|| crate::vm::method_lookup::module_extended_method(owner, name))
}

impl VirtualMachine {
    /// `__guard_fast_path_class_method__(owner, name)`: records the method
    /// `owner` answers `name` with itself, as one a fast path stands in for.
    pub(crate) fn guard_fast_path_class_method(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (Some(View::Class(owner) | View::Module(owner)), Some(name)) =
            (arguments.first().view(), arguments.get(1))
        else {
            return Ok(Object::Nil);
        };
        let name = self.coerce_method_name(name, "__guard_fast_path_class_method__", position)?;
        if let Some(original) = class_level_method(owner, &name) {
            self.fast_path_class_originals
                .push((Rc::clone(owner), name, original));
        }
        Ok(Object::Nil)
    }

    /// `__guard_fast_matcher__(kind, matcher_class, *method_names)`: records
    /// `matcher_class` as the one `kind` names, and its methods as ones the
    /// native match stands in for.
    pub(crate) fn guard_fast_matcher(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (Some(View::Symbol(kind)), Some(View::Class(matcher_class))) =
            (arguments.first().view(), arguments.get(1).view())
        else {
            return Ok(Object::Nil);
        };
        for name in &arguments[2..] {
            let name = self.coerce_method_name(name, "__guard_fast_matcher__", position)?;
            if let Some(original) = matcher_class.find_method(&name) {
                self.fast_path_originals
                    .entry(name)
                    .or_default()
                    .push(original);
            }
        }
        self.fast_matchers.push((
            kind.to_string(),
            Rc::clone(matcher_class),
            arguments[2..].to_vec(),
        ));
        Ok(Object::Nil)
    }

    /// Whether each class-level method recorded is still the one answering,
    /// checked again only after the method state moved.
    fn class_methods_as_recorded(&mut self) -> bool {
        let state = crate::class::method_state();
        if let Some((checked_under, held)) = self.fast_path_class_check
            && checked_under == state
        {
            return held;
        }
        let held = self
            .fast_path_class_originals
            .iter()
            .all(|(owner, name, original)| {
                class_level_method(owner, name)
                    .is_some_and(|answered| Rc::ptr_eq(&answered, original))
            });
        self.fast_path_class_check = Some((state, held));
        held
    }

    /// `__expect_to__(actual, matcher, message, block, matchers_module,
    /// target_class, handler)`:
    /// `expect(actual).to(matcher)` for the matchers Rex matches natively.
    /// Answers what the match answers when it passes, and false when it
    /// fails, having set `@actual`, `last_matcher`, and
    /// `last_expectation_handler` as the Ruby steps do. Answers nil, having
    /// done nothing, when the Ruby steps have to run: the fast paths are
    /// off, a message or block was given, the matcher is not one of those,
    /// or a method they read past was redefined.
    pub(crate) fn expect_to(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (
            Some(actual),
            Some(View::Instance(matcher)),
            Some(View::Nil),
            Some(View::Nil),
            Some(View::Module(matchers_module)),
            Some(View::Class(target_class)),
            Some(handler),
        ) = (
            arguments.first(),
            arguments.get(1).view(),
            arguments.get(2).view(),
            arguments.get(3).view(),
            arguments.get(4).view(),
            arguments.get(5).view(),
            arguments.get(6),
        )
        else {
            return Ok(Object::Nil);
        };
        if !self.fast_paths_enabled
            || !self.answers_as_recorded(target_class, "enforce_value_expectation")
            || !self.answers_as_recorded(target_class, "with_source_location")
        {
            return Ok(Object::Nil);
        }
        let matcher_class = Rc::clone(&matcher.borrow().class);
        let Some((kind, method_names)) = self
            .fast_matchers
            .iter()
            .find(|(_, class, _)| Rc::ptr_eq(class, &matcher_class))
            .map(|(kind, _, names)| (kind.clone(), names.clone()))
        else {
            return Ok(Object::Nil);
        };
        for name in &method_names {
            let name = self.coerce_method_name(name, "__expect_to__", position)?;
            if !self.answers_as_recorded(&matcher_class, &name) {
                return Ok(Object::Nil);
            }
        }
        if !self.class_methods_as_recorded() {
            return Ok(Object::Nil);
        }
        let expected = matcher.borrow().get_var("expected").cloned();
        let matched = match (kind.as_str(), expected.view()) {
            ("eq", Some(expected)) => {
                self.send_to_object(actual.clone(), "==", vec![expected.to_object()], position)?
            }
            ("equal", Some(expected)) => self.send_to_object(
                actual.clone(),
                "equal?",
                vec![expected.to_object()],
                position,
            )?,
            ("be_truthy", _) => Object::Bool(actual.is_truthy()),
            ("be_nil", _) => self.send_to_object(actual.clone(), "nil?", Vec::new(), position)?,
            ("match", Some(expected @ View::Regex(_, _)))
                if matches!(actual.view(), View::String(_)) =>
            {
                // RSpec's Match holds `@expected_captures` as nil until
                // `with_captures` names some.
                if matcher
                    .borrow()
                    .get_var("expected_captures")
                    .is_some_and(|captures| !matches!(captures.view(), View::Nil))
                {
                    return Ok(Object::Nil);
                }
                let matched = self.send_to_object(
                    expected.to_object(),
                    "===",
                    vec![actual.clone()],
                    position,
                )?;
                Object::Bool(matched.is_truthy())
            }
            ("include", _) => match self.included_natively(actual, matcher, position)? {
                Some(included) => Object::Bool(included),
                None => return Ok(Object::Nil),
            },
            _ => return Ok(Object::Nil),
        };
        if kind != "include" {
            matcher
                .borrow_mut()
                .set_var("actual".to_string(), actual.clone());
        }
        matchers_module.set_class_var("@last_expectation_handler", handler.clone());
        matchers_module.set_class_var("@last_matcher", Object::Instance(Rc::clone(matcher)));
        Ok(if matched.is_truthy() {
            matched
        } else {
            Object::Bool(false)
        })
    }

    /// Whether every item of an `include` matcher is in `actual`, an Array
    /// or a String, by `include?`, having set what the Ruby steps set. None
    /// when the Ruby steps have to decide: a count was asked for, `actual`
    /// is something else, or an item is missing and only `===` could still
    /// find it.
    fn included_natively(
        &mut self,
        actual: &Object,
        matcher: &Rc<std::cell::RefCell<crate::object::Instance>>,
        position: Position,
    ) -> Result<Option<bool>, MetorexError> {
        let held = |name: &str| matcher.borrow().get_var(name).cloned();
        if held("expected_count").is_some_and(|count| count.is_truthy()) {
            return Ok(None);
        }
        let viewed = held("expecteds");
        let Some(View::Array(expecteds)) = viewed.view() else {
            return Ok(None);
        };
        let expecteds = expecteds.borrow().clone();
        match actual.view() {
            View::Array(_) => {}
            View::String(_)
                if expecteds
                    .iter()
                    .all(|item| matches!(item.view(), View::String(_))) => {}
            _ => return Ok(None),
        }
        for item in expecteds {
            let found = self.send_to_object(actual.clone(), "include?", vec![item], position)?;
            if !found.is_truthy() {
                return Ok(None);
            }
        }
        let mut matcher = matcher.borrow_mut();
        matcher.set_var("actual".to_string(), actual.clone());
        matcher.set_var("divergent_items".to_string(), Object::empty_array());
        Ok(Some(true))
    }
}

impl VirtualMachine {
    /// `__mock_space_idle__(space, space_class)`: whether the mock space of
    /// the example running now holds nothing to verify or take down, so
    /// verifying it and taking it down would each walk empty tables. True
    /// for no space at all. False when the fast paths are off, or
    /// `verify_all` or `reset_all` is not Rex's own.
    pub(crate) fn mock_space_idle(&mut self, arguments: &[Object]) -> Result<Object, MetorexError> {
        let (Some(space), Some(View::Class(space_class))) =
            (arguments.first(), arguments.get(1).view())
        else {
            return Ok(Object::Bool(false));
        };
        if !self.fast_paths_enabled
            || !self.answers_as_recorded(space_class, "verify_all")
            || !self.answers_as_recorded(space_class, "reset_all")
        {
            return Ok(Object::Bool(false));
        }
        let space = match space.view() {
            View::Nil => return Ok(Object::Bool(true)),
            View::Instance(space) if Rc::ptr_eq(&space.borrow().class, space_class) => space,
            _ => return Ok(Object::Bool(false)),
        };
        let space = space.borrow();
        let empty = |name: &str| match space.get_var(name).view() {
            Some(View::Dict(table)) => table.borrow().is_empty(),
            Some(View::Array(list)) => list.borrow().is_empty(),
            _ => false,
        };
        Ok(Object::Bool(
            empty("proxies") && empty("any_instance_recorders") && empty("constant_mutators"),
        ))
    }
}
