//! The interpreter's side of `TracePoint`: which traces are switched on, and
//! calling them as the events they asked for happen.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::object::{OptionView, View};
use crate::vm::VirtualMachine;
use std::rc::Rc;

/// A method or block body that is running, as a trace sees it.
pub(crate) struct RunningCode {
    /// The places the body was written within, ending with its own.
    pub(crate) within: crate::object::CodePlaces,
    /// The line the body last ran, which a `return` or `b_return` event
    /// reports.
    pub(crate) line: usize,
    /// Whether the body left through a `return`, whose line a `return`
    /// event names rather than the line the definition ends on.
    pub(crate) returned_explicitly: bool,
    /// How many native calls were running when the body started, put back
    /// when it ends.
    native_calls_before: usize,
}

impl VirtualMachine {
    /// Note that a method or block body starts running. Code written in Ruby
    /// fires `c_call` events for the native methods it calls, even when a
    /// native method is what called it.
    pub(crate) fn enter_running_code(&mut self, within: crate::object::CodePlaces, line: usize) {
        let native_calls_before = std::mem::take(&mut self.native_calls_running);
        self.running_code.push(RunningCode {
            within,
            line,
            returned_explicitly: false,
            native_calls_before,
        });
    }

    /// Note that the body `enter_running_code` noted has finished.
    pub(crate) fn leave_running_code(&mut self) {
        if let Some(finished) = self.running_code.pop() {
            self.native_calls_running = finished.native_calls_before;
        }
    }
}

impl VirtualMachine {
    /// Take a tracepoint's own record of whether it is on and bring the
    /// interpreter's list into line with it.
    pub(crate) fn register_tracepoint(&mut self, tracepoint: &Object) {
        let switched_on = matches!(
            self.read_instance_var(tracepoint, "enabled").view(),
            Some(View::Bool(true))
        );
        self.tracepoints
            .retain(|held| !same_tracepoint(held, tracepoint));
        if switched_on {
            crate::vm::jit::assumptions_changed();
            self.tracepoints.push(tracepoint.clone());
        }
    }

    /// Fire a `c_call` or `c_return` event for a method the interpreter
    /// carries natively, which Ruby reports as a method written in C.
    pub(crate) fn fire_native_event(
        &mut self,
        event: &str,
        method_name: &str,
        receiver: &Object,
        value: Option<&Object>,
        position: Position,
    ) -> Result<(), MetorexError> {
        if self.tracepoints.is_empty()
            || self.tracing
            || position.prelude
            || self.native_calls_running > 0
        {
            return Ok(());
        }
        self.fire_native_event_now(event, method_name, receiver, value, position)
    }

    /// Fire a native method's event whatever is running now, for one whose
    /// conditions were settled when its call began.
    fn fire_native_event_now(
        &mut self,
        event: &str,
        method_name: &str,
        receiver: &Object,
        value: Option<&Object>,
        position: Position,
    ) -> Result<(), MetorexError> {
        let class = self.builtins().class_of(receiver);
        let mut extra = vec![
            ("method_id", Object::symbol(method_name.to_string())),
            ("callee_id", Object::symbol(method_name.to_string())),
            ("defined_class", Object::Class(class)),
            ("self", receiver.clone()),
        ];
        if let Some(value) = value {
            extra.push(("return_value", value.clone()));
        }
        self.fire_event(event, position, extra)
    }

    /// Whether a tracepoint handler is running, which is what
    /// `TracePoint.allow_reentry` is only allowed inside.
    pub(crate) fn is_tracing(&self) -> bool {
        self.tracing
    }

    /// Switch tracing off and answer what it was, so a block can fire events
    /// of its own from inside a handler. The line already reported is carried
    /// along too: the lines the block runs are its own, and the line the
    /// handler was called from still counts as reported.
    pub(crate) fn take_tracing(&mut self) -> (bool, Option<(String, usize)>) {
        (
            std::mem::replace(&mut self.tracing, false),
            self.traced_line.take(),
        )
    }

    /// Put the tracing flag and the reported line back to what
    /// `take_tracing` found.
    pub(crate) fn restore_tracing(&mut self, held: (bool, Option<(String, usize)>)) {
        self.tracing = held.0;
        self.traced_line = held.1;
    }

    /// Fire a `:line` event for the statement about to run. A statement on a
    /// line already traced fires nothing, so one line is one event however
    /// many statements share it.
    pub(crate) fn fire_line_event(&mut self, position: Position) -> Result<(), MetorexError> {
        // The core library stands in for Ruby's C code, which a trace never
        // sees.
        if self.tracepoints.is_empty() || position.prelude {
            return Ok(());
        }
        if let Some(running) = self.running_code.last_mut() {
            running.line = position.line;
        }
        if self.tracing {
            return Ok(());
        }
        // One line reports once, and the same line number in another file is
        // a line of its own.
        let reached = (self.traced_path(), position.line);
        if self.traced_line.as_ref() == Some(&reached) {
            return Ok(());
        }
        self.traced_line = Some(reached);
        self.fire_event("line", position, Vec::new())
    }

    /// Hand one event to every tracepoint that asked for it.
    /// Whether `fire_event` would hand an event to a trace, so the caller
    /// builds what the event carries only then.
    pub(crate) fn events_wanted(&self) -> bool {
        !self.tracepoints.is_empty() && !self.tracing
    }

    pub(crate) fn fire_event(
        &mut self,
        event: &str,
        position: Position,
        extra: Vec<(&str, Object)>,
    ) -> Result<(), MetorexError> {
        if !self.events_wanted() {
            return Ok(());
        }
        // Everything from here on runs the trace's own code, which never
        // fires events of its own.
        self.tracing = true;
        // A `call` event fires before the method called has taken the block
        // it was given, and the calls made to the trace would take it first.
        let held_block = self.pending_block.take();
        let held_from_ampersand = self.pending_block_from_ampersand;
        let outcome = self.deliver_event(event, position, &extra);
        self.pending_block = held_block;
        self.pending_block_from_ampersand = held_from_ampersand;
        self.tracing = false;
        outcome
    }

    /// Hand one event to each listening tracepoint. Called with tracing
    /// already marked, so nothing here fires again.
    fn deliver_event(
        &mut self,
        event: &str,
        position: Position,
        extra: &[(&str, Object)],
    ) -> Result<(), MetorexError> {
        // A trace aimed at one method or block runs after every other, and
        // the one switched on last runs first.
        let (mut listening, targeted): (Vec<Object>, Vec<Object>) =
            self.tracepoints.iter().cloned().partition(|held| {
                matches!(
                    self.read_instance_var(held, "target_place").view(),
                    None | Some(View::Nil)
                )
            });
        listening.extend(targeted.into_iter().rev());
        let name = Object::symbol(event.to_string());
        for tracepoint in listening {
            let wants = self.send_to_object(
                tracepoint.clone(),
                "__wants__",
                vec![name.clone()],
                position,
            )?;
            if !matches!(wants.view(), View::Bool(true)) {
                continue;
            }
            let details = self.event_details(event, position, extra);
            let accepted = self.send_to_object(
                tracepoint.clone(),
                "__accepts__",
                vec![details.clone()],
                position,
            )?;
            if accepted.is_truthy() {
                self.run_trace_handler(&tracepoint, details, position)?;
            }
        }
        Ok(())
    }

    /// Runs a tracepoint's block with `details` as the event it reads. The
    /// block is called from here rather than from the core library, so no
    /// frame stands between it and the code that fired the event.
    fn run_trace_handler(
        &mut self,
        tracepoint: &Object,
        details: Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let held = self.read_instance_var(tracepoint, "handling");
        write_instance_var(tracepoint, "handling", details);
        let block = self
            .read_instance_var(tracepoint, "block")
            .unwrap_or(Object::Nil);
        let outcome = self.send_to_object(block, "call", vec![tracepoint.clone()], position);
        write_instance_var(tracepoint, "handling", held.unwrap_or(Object::Nil));
        outcome.map(|_| ())
    }

    /// What the event says about itself, as the Hash a tracepoint reads its
    /// answers out of.
    fn event_details(
        &mut self,
        event: &str,
        position: Position,
        extra: &[(&str, Object)],
    ) -> Object {
        let mut entries = indexmap::IndexMap::new();
        entries.insert("event".to_string(), Object::symbol(event.to_string()));
        entries.insert("lineno".to_string(), Object::Int(position.line as i64));
        entries.insert("path".to_string(), Object::string(self.traced_path()));
        entries.insert(
            "self".to_string(),
            self.environment().get("self").unwrap_or(Object::Nil),
        );
        for (name, value) in extra {
            entries.insert((*name).to_string(), value.clone());
        }
        // Code written in Ruby is read through a binding of the scope it
        // runs in, which a C method has none of.
        if !entries.contains_key("binding")
            && !matches!(event, "c_call" | "c_return")
            && let Ok(binding) = self.kernel_binding(position)
        {
            entries.insert("binding".to_string(), binding);
        }
        // A line or a raise inside a method names that method and the class
        // it was written on.
        if matches!(event, "line" | "raise")
            && !entries.contains_key("method_id")
            && let Some((callee, defined)) = self.enclosing_method_names()
        {
            entries.insert("method_id".to_string(), Object::symbol(&*defined));
            entries.insert("callee_id".to_string(), Object::symbol(&*callee));
            if let Some((Some(owner), _)) = self.enclosing_method_owner()
                && let Some(class) = self.resolve_qualified_constant(&owner)
            {
                entries.insert("defined_class".to_string(), class);
            }
        }
        let within = self
            .running_code
            .last()
            .map(|running| {
                running
                    .within
                    .iter()
                    .map(|(file, line)| {
                        Object::array(vec![
                            Object::string(file.clone()),
                            Object::Int(*line as i64),
                        ])
                    })
                    .collect()
            })
            .unwrap_or_default();
        entries.insert("__within__".to_string(), Object::array(within));
        Object::Dict(Rc::new(std::cell::RefCell::new(entries)))
    }

    /// The file the code running now was written in, as `path` reports it:
    /// the main script the way it was given, as `__FILE__` names it.
    fn traced_path(&self) -> String {
        if let Some(file) = &self.current_source_file {
            return file.clone();
        }
        match self.reported_current_file() {
            Some(path) => path.to_string_lossy().to_string(),
            None => "-e".to_string(),
        }
    }

    /// One instance variable off an object, where it has one.
    fn read_instance_var(&self, holder: &Object, name: &str) -> Option<Object> {
        let View::Instance(instance) = holder.view() else {
            return None;
        };
        instance.borrow().get_var(name).cloned()
    }
}

/// Sets an instance variable of a tracepoint, which is always an instance.
fn write_instance_var(holder: &Object, name: &str, value: Object) {
    if let View::Instance(instance) = holder.view() {
        instance.borrow_mut().set_var(name.to_string(), value);
    }
}

/// Whether two references name the same tracepoint.
fn same_tracepoint(one: &Object, other: &Object) -> bool {
    match (one.view(), other.view()) {
        (View::Instance(a), View::Instance(b)) => Rc::ptr_eq(a, b),
        _ => false,
    }
}

impl VirtualMachine {
    /// Fire a `:call` or `:return` event for a method written in Ruby. The
    /// method's own name, the class it was found on, and what it answered are
    /// what a trace reads back.
    pub(crate) fn fire_method_event(
        &mut self,
        event: &str,
        method_name: &str,
        method: &Rc<crate::object::Method>,
        found_on: &Rc<crate::class::Class>,
        value: Option<&Object>,
        position: Position,
    ) -> Result<(), MetorexError> {
        if self.tracepoints.is_empty() || self.tracing || position.prelude {
            return Ok(());
        }
        // The core library stands in for Ruby's C code, so a method written
        // there is reported as a C method, at the place it was called from.
        let written_in_core = reported_as_c(method);
        // One a native method calls is C code calling C code, which fires
        // nothing. The method's body has started by now, so the count is the
        // one it set aside.
        let called_natively = self
            .running_code
            .last()
            .is_some_and(|running| running.native_calls_before > 0);
        if written_in_core && called_natively {
            return Ok(());
        }
        // MRI writes TracePoint#enable and #disable in Ruby, in its
        // trace_point.rb. A trace hears no event of `enable`, which switches
        // it on, and of `disable`, which switches it off, only the call and
        // the line it runs first. C switching a trace off calls neither.
        if found_on.name() == "TracePoint" && matches!(method_name, "enable" | "disable") {
            if method_name == "enable" || event != "call" || called_natively {
                return Ok(());
            }
            let place = |line: usize| {
                vec![
                    ("method_id", Object::symbol("disable")),
                    ("callee_id", Object::symbol("disable")),
                    ("defined_class", Object::Class(Rc::clone(found_on))),
                    ("path", Object::string("<internal:trace_point>")),
                    ("lineno", Object::Int(line as i64)),
                    ("parameters", Object::empty_array()),
                ]
            };
            self.fire_event("call", position, place(TRACE_POINT_DISABLE_LINE))?;
            return self.fire_event("line", position, place(TRACE_POINT_DISABLE_FIRST_LINE));
        }
        // A call the native methods missed has had its `c_call` fired.
        if written_in_core
            && event == "call"
            && self
                .unanswered_native_call
                .take()
                .is_some_and(|held| held == method_name)
        {
            return Ok(());
        }
        let (event, position) = match (written_in_core, event) {
            (true, "call") => ("c_call", position),
            (true, _) => ("c_return", position),
            // A method's call names the line it was defined on, and its
            // return the line it ran last.
            (false, "call") => (
                event,
                match &method.source_location {
                    Some(written) => Position::new(written.line, written.column, written.offset),
                    None => position,
                },
            ),
            // A method's return names the line it returned from, or the
            // line its definition ends on when the body ran to its end.
            (false, _) => (
                event,
                match (self.running_code.last(), method.end_line) {
                    (Some(running), Some(end)) if !running.returned_explicitly => {
                        Position::new(end, 0, 0)
                    }
                    (Some(running), _) => Position::new(running.line, 0, 0),
                    (None, _) => position,
                },
            ),
        };
        let plain = method_name
            .strip_prefix("__class__")
            .unwrap_or(method_name)
            .to_string();
        // A method reached through an alias is named by what it was defined
        // as, and called by the alias.
        let defined = method
            .provenance
            .original_name
            .clone()
            .unwrap_or_else(|| plain.clone());
        let mut extra = vec![
            ("method_id", Object::symbol(defined)),
            ("callee_id", Object::symbol(plain)),
            // Ruby names the class the method was written on, which is not
            // always the one the call found it through.
            ("defined_class", {
                // A module the method was written on stands as a module, not
                // as a class, so the object a trace reads is the same one the
                // name reaches.
                let owner = method
                    .owner_class
                    .clone()
                    .unwrap_or_else(|| Rc::clone(found_on));
                if owner.is_module() {
                    Object::Module(owner)
                } else {
                    Object::Class(owner)
                }
            }),
            // A trace reads the parameters a method declared off either of
            // the events that name the method.
            (
                "parameters",
                crate::vm::native_methods::method_parameter_list(method),
            ),
        ];
        if let Some(value) = value {
            extra.push(("return_value", value.clone()));
        }
        if event == "return"
            && let Some(held) = self.traced_binding.take()
        {
            extra.push(("binding", held));
        }
        self.fire_event(event, position, extra)
    }
}

/// Whether a trace reports `method` as one Ruby writes in C: one the core
/// library writes, or one an attribute call made.
pub(crate) fn reported_as_c(method: &crate::object::Method) -> bool {
    method
        .body
        .first()
        .is_some_and(|held| held.position().prelude)
        || made_by_an_attribute_call(method)
}

/// The line MRI 4.0.7's trace_point.rb defines `TracePoint#disable` on, and
/// the line of its body it runs first.
const TRACE_POINT_DISABLE_LINE: usize = 297;
const TRACE_POINT_DISABLE_FIRST_LINE: usize = 299;

/// The unary operators MRI 4.0.7 writes in Ruby in its numeric.rb, by the
/// class each is defined on, with the lines a trace reports for its call
/// and for its return.
pub(crate) const NUMERIC_UNARY_METHODS: &[(&str, &str, usize, usize)] = &[
    ("Integer", "-@", 104, 107),
    ("Numeric", "+@", 89, 91),
    ("Float", "-@", 389, 392),
];

impl VirtualMachine {
    /// Run `operation`, a unary operator on `receiver`, while a TracePoint
    /// listens. One MRI writes in numeric.rb fires `call` and `return` from
    /// there, and any other one the receiver answers natively fires
    /// `c_call` and `c_return`.
    pub(crate) fn traced_unary_operator(
        &mut self,
        receiver: &Object,
        name: &str,
        position: Position,
        operation: impl FnOnce(&mut Self) -> Result<Object, MetorexError>,
    ) -> Result<Object, MetorexError> {
        let class = match receiver.view() {
            View::Int(_) | View::BigInt(_) => Some("Integer"),
            View::Float(_) => Some("Float"),
            _ => None,
        };
        let written = class.and_then(|class| {
            NUMERIC_UNARY_METHODS.iter().find(|(owner, method, _, _)| {
                *method == name && (*owner == class || *owner == "Numeric")
            })
        });
        let Some((owner, _, call_line, return_line)) = written.copied() else {
            return self.traced_native_operation(receiver, name, position, operation);
        };
        let defined_class = self.globals().get(owner).unwrap_or(Object::Nil);
        // The event names the line in numeric.rb, while the line running
        // stays the one that called the operator.
        let extra = |line: usize, value: Option<Object>| {
            let mut extra = vec![
                ("method_id", Object::symbol(name)),
                ("callee_id", Object::symbol(name)),
                ("defined_class", defined_class.clone()),
                ("path", Object::string("<internal:numeric>")),
                ("lineno", Object::Int(line as i64)),
            ];
            if let Some(value) = value {
                extra.push(("return_value", value));
            }
            extra
        };
        self.fire_event("call", position, extra(call_line, None))?;
        let answered = operation(self)?;
        self.fire_event(
            "return",
            position,
            extra(return_line, Some(answered.clone())),
        )?;
        Ok(answered)
    }
}

/// Whether `attr_reader`, `attr_writer` or `attr_accessor` made the method,
/// which Ruby writes in C and so reports through `c_call` and `c_return`.
/// Those methods have no source of their own and do nothing but read or
/// write their instance variable.
fn made_by_an_attribute_call(method: &crate::object::Method) -> bool {
    use crate::ast::{Expression, Statement};
    if method.source_location.is_some() {
        return false;
    }
    match &method.body[..] {
        [
            Statement::Return {
                value: Some(Expression::InstanceVariable { .. }),
                ..
            },
        ] => method.parameters.is_empty(),
        [
            Statement::Assignment {
                target: Expression::InstanceVariable { .. },
                value: Expression::Identifier { name, .. },
                ..
            },
        ] => name == crate::object::UNNAMED_PARAMETER,
        _ => false,
    }
}
