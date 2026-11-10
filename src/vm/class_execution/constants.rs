// Finding the constant a name stands for, from the scope the name
// was written in outwards.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// Whether an unqualified constant may fall through to the top level.
    /// Ruby reaches top-level constants because they are Object's, so a class
    /// body whose class does not descend from Object never sees them.
    pub(crate) fn lexical_scope_reaches_top_level(&self) -> bool {
        // A block run as a class body, as `class_exec` runs one, reads
        // constants where it was written rather than in the class it runs on.
        if let Some((depth, home)) = self.constant_homes.last()
            && *depth == self.def_scope_stack.len()
        {
            return home.as_ref().is_none_or(|home| {
                crate::vm::method_invocation::descends_from(home, "Object") || home.is_module()
            });
        }
        // Inside a method body the lexical chain is the nesting captured
        // where the method was defined, innermost first. That holds over a
        // class body the call was made from, unless a class body opened
        // inside the method since.
        let method_nesting = || {
            self.method_nesting_stack
                .last()
                .and_then(|nesting| nesting.first())
                .cloned()
        };
        let in_method_body = matches!(
            self.written_in_method(),
            Some((depth, _)) if depth == self.def_scope_stack.len()
        );
        let innermost = if in_method_body {
            // A method written at the top level reads the top level.
            let Some(written_in) = method_nesting() else {
                return true;
            };
            Some(written_in)
        } else {
            self.def_scope_stack.last().cloned().or_else(method_nesting)
        };
        let Some(innermost) = innermost else {
            return true;
        };
        // Object open further out in the nesting reaches them lexically.
        if !in_method_body
            && self
                .def_scope_stack
                .iter()
                .any(|open| open.ruby_name_is("Object"))
        {
            return true;
        }
        // A module has no superclass chain to judge by, so it stays open.
        let mut cursor = Some(innermost);
        while let Some(current) = cursor {
            if current.name() == "Object" {
                return true;
            }
            if current.name() == "BasicObject" {
                return false;
            }
            cursor = current.superclass();
        }
        true
    }

    /// Resolve a bare constant name using Ruby-like lexical scoping: walk
    /// the enclosing class/module stack (innermost first), then check the
    /// local environment, then globals. Used by `include`/`extend` inside
    /// class bodies so nested modules can reference siblings without
    /// fully-qualifying them. Also handles qualified names (`A::B::C`).
    pub(crate) fn resolve_constant_in_scope(&self, name: &str) -> Option<Object> {
        if name.contains("::") {
            let mut parts = name.split("::");
            let head = parts.next()?;
            let mut current = self.resolve_constant_in_scope(head)?;
            for part in parts {
                let class_rc = match current.view() {
                    View::Class(c) | View::Module(c) => Rc::clone(c),
                    _ => return None,
                };
                current = class_rc.get_class_var(part)?;
            }
            return Some(current);
        }
        // The modules open where the code was written, and the ancestors of
        // the innermost one, as a constant read anywhere else finds them.
        if let Some(value) = self.constant_before_object(name) {
            return Some(value);
        }
        // A still-being-defined enclosing scope resolves to itself when
        // referenced by its own simple name (e.g. `ModuleSpecs::Alias`
        // inside `module ModuleSpecs; class Allonym; include ... end; end`
        // before ModuleSpecs has been bound in globals).
        if let Some(enclosing) = self
            .def_scope_stack
            .iter()
            .rev()
            .find(|held| held.name() == name)
        {
            return Some(scope_object(enclosing));
        }
        self.environment()
            .get(name)
            .or_else(|| self.globals().constant(name))
    }

    /// Like `resolve_constant_in_scope`, but triggers autoload on any
    /// segment along a qualified `A::B::C` walk where the next part is only
    /// registered as an autoload. Used by class/module statement positions
    /// that should fire pending autoloads (e.g. `class X < A::B::Auto`).
    pub(crate) fn resolve_constant_with_autoload(
        &mut self,
        name: &str,
    ) -> Result<Option<Object>, MetorexError> {
        if name.contains("::") {
            let mut parts = name.split("::");
            let head = match parts.next() {
                Some(h) => h,
                None => return Ok(None),
            };
            let mut current = match self.resolve_constant_with_autoload(head)? {
                Some(v) => v,
                None => return Ok(None),
            };
            for part in parts {
                let class_rc = match current.view() {
                    View::Class(c) | View::Module(c) => Rc::clone(c),
                    _ => return Ok(None),
                };
                if let Some(v) = class_rc.get_class_var(part) {
                    current = v;
                } else if let Some(v) = self.try_autoload_constant(&class_rc, part)? {
                    current = v;
                } else {
                    return Ok(None);
                }
            }
            return Ok(Some(current));
        }
        // Each module around the code, innermost first, answers with a
        // constant of its own or one it autoloads, which is loaded now, before
        // a module further out is asked. A block's modules are the ones
        // around where it was written, as in an `included do` block a class
        // runs through `class_eval`.
        // The class and module bodies running now come after those, as
        // the scopes a body opened in a file an autoload loads does, so
        // `class ArrayBuffer < Array` in temple's Generators loads that
        // module's own Array ahead of the top-level one.
        let mut scopes = self.lexical_nesting();
        for open in self.def_scope_stack.iter().rev() {
            if !scopes.iter().any(|held| Rc::ptr_eq(held, open)) {
                scopes.push(Rc::clone(open));
            }
        }
        for scope in scopes {
            if let Some(found) = scope.get_class_var(name) {
                return Ok(Some(found));
            }
            if scope.lookup_autoload(name).is_some()
                && let Some(loaded) = self.try_autoload_constant(&scope, name)?
            {
                return Ok(Some(loaded));
            }
        }
        if let Some(found) = self.resolve_constant_in_scope(name) {
            return Ok(Some(found));
        }
        if let Some(View::Class(object)) = self.globals().get("Object").view() {
            let object = object.clone();
            if object.lookup_autoload(name).is_some()
                && let Some(loaded) = self.try_autoload_constant(&object, name)?
            {
                return Ok(Some(loaded));
            }
        }
        Ok(None)
    }

    /// Resolve a `Foo::Bar::Baz` qualified constant by walking from the
    /// outermost name through nested module/class constants.
    pub(crate) fn resolve_qualified_constant(&self, qualified: &str) -> Option<Object> {
        let mut parts = qualified.split("::");
        let head = parts.next()?;
        let mut current = self
            .environment()
            .get(head)
            .or_else(|| self.globals().get(head))?;
        for part in parts {
            let class_rc = match current.view() {
                View::Class(c) | View::Module(c) => Rc::clone(c),
                _ => return None,
            };
            current = class_rc.get_class_var(part)?;
        }
        Some(current)
    }
}

/// A class or module body's scope as the value its name stands for.
pub(crate) fn scope_object(scope: &Rc<Class>) -> Object {
    if scope.is_module() {
        Object::Module(Rc::clone(scope))
    } else {
        Object::Class(Rc::clone(scope))
    }
}
