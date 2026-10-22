// The method table, the mixin chains, and the lookups over them.

use super::*;
use crate::object::{OptionView, View};

thread_local! {
    /// A number every change to a method table, a mixin chain or a
    /// singleton's attachment moves on, so a lookup cached under an earlier
    /// one is not trusted.
    static METHOD_STATE: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Mark every cached method lookup out of date.
pub(crate) fn method_state_changed() {
    METHOD_STATE.with(|state| state.set(state.get() + 1));
}

/// `method_state_changed`, for a change to the methods named `name`. One of
/// the operators machine code applies to Integers itself moves the JIT's
/// epoch too.
pub(crate) fn method_named_changed(name: &str) {
    method_state_changed();
    if crate::vm::specialized::OPERATOR_NAMES.contains(&name) {
        crate::vm::jit::assumptions_changed();
    }
}

/// `method_state_changed`, for `module` joining a mixin or prepend chain.
/// A module answering one of the operators moves the JIT's epoch too.
fn module_joined(module: &Class) {
    method_state_changed();
    if crate::vm::specialized::OPERATOR_NAMES
        .iter()
        .any(|name| module.find_method(name).is_some())
    {
        crate::vm::jit::assumptions_changed();
    }
}

/// The number `method_state_changed` moves on: the class lookup caches and
/// the fast paths' guards hold an answer only while it stays where it was.
pub fn method_state() -> u64 {
    METHOD_STATE.with(|state| state.get())
}

/// Whether a fast path may run, which turns on a check of the methods it
/// stands in for, such as that `eq` is still Rex's own. The check runs again
/// only after a method table, a mixin chain, or a singleton's attachment
/// changed, which is what `method_state` counts.
#[derive(Debug, Default)]
pub struct MethodGuard {
    /// The state the last check ran under, or None before the first.
    checked_under: std::cell::Cell<Option<u64>>,
    holds: std::cell::Cell<bool>,
}

impl MethodGuard {
    pub const fn new() -> Self {
        Self {
            checked_under: std::cell::Cell::new(None),
            holds: std::cell::Cell::new(false),
        }
    }

    /// What `check` answered under the current method state, running it
    /// only when the state moved since it last ran.
    pub fn holds(&self, check: impl FnOnce() -> bool) -> bool {
        let state = method_state();
        if self.checked_under.get() != Some(state) {
            self.holds.set(check());
            // The check may define a method itself, which moves the state
            // past the one it ran under, so the state is read after it.
            self.checked_under.set(Some(method_state()));
        }
        self.holds.get()
    }
}

impl Class {
    /// Define or replace a method on this class.
    pub fn define_method(&self, name: impl Into<String>, method: Rc<Method>) {
        let name = name.into();
        method_named_changed(&name);
        if name.starts_with("__class__") {
            self.holds_class_methods.set(true);
        }
        self.methods.borrow_mut().insert(name, method);
    }

    /// Set `holds_class_methods` and `holds_extended_copies` from what the
    /// tables hold, after they were copied in whole.
    pub(crate) fn note_held_names(&self) {
        self.holds_class_methods.set(
            self.methods
                .borrow()
                .keys()
                .any(|name| name.starts_with("__class__")),
        );
        self.holds_extended_copies.set(
            self.class_variables
                .borrow()
                .keys()
                .any(|name| name.starts_with("__ext__")),
        );
    }

    /// Determine whether this class defines a method (without checking superclasses).
    pub fn has_own_method(&self, name: &str) -> bool {
        self.methods.borrow().contains_key(name)
    }

    /// Add an included module to the mixin chain.
    /// The module is prepended to the list so the most-recently-included module
    /// is searched first (Ruby's MRO).
    pub fn add_mixin(&self, module: Rc<Class>) {
        module_joined(&module);
        self.mixins.borrow_mut().insert(0, module);
    }

    /// Whether any module is mixed into this class or module.
    pub fn has_mixins(&self) -> bool {
        !self.mixins.borrow().is_empty()
    }

    /// Snapshot of the mixin chain (most-recently-included first).
    pub fn mixin_chain(&self) -> Vec<Rc<Class>> {
        self.mixins.borrow().clone()
    }

    /// Add a prepended module, ahead of the class's own method table.
    pub fn add_prepend(&self, module: Rc<Class>) {
        module_joined(&module);
        self.prepends.borrow_mut().insert(0, module);
    }

    /// Whether any module is prepended to this class or module.
    pub fn has_prepends(&self) -> bool {
        !self.prepends.borrow().is_empty()
    }

    /// Snapshot of the prepend chain (most-recently-prepended first).
    pub fn prepend_chain(&self) -> Vec<Rc<Class>> {
        self.prepends.borrow().clone()
    }

    /// The prepend chain expanded through each prepended module's own
    /// prepends and includes, in lookup order.
    pub fn transitive_prepends(&self) -> Vec<Rc<Class>> {
        let mut chain = Vec::new();
        let mut seen: Vec<*const Class> = Vec::new();
        for prepended in self.prepend_chain() {
            let ptr = Rc::as_ptr(&prepended);
            if seen.contains(&ptr) {
                continue;
            }
            seen.push(ptr);
            for nested in prepended.transitive_prepends() {
                let nested_ptr = Rc::as_ptr(&nested);
                if !seen.contains(&nested_ptr) {
                    seen.push(nested_ptr);
                    chain.push(nested);
                }
            }
            chain.push(Rc::clone(&prepended));
            for mixin in prepended.transitive_mixins() {
                let mixin_ptr = Rc::as_ptr(&mixin);
                if !seen.contains(&mixin_ptr) {
                    seen.push(mixin_ptr);
                    chain.push(mixin);
                }
            }
        }
        chain
    }

    /// True when `module` has already been prepended here.
    pub fn has_prepend(&self, module: &Rc<Class>) -> bool {
        self.prepends
            .borrow()
            .iter()
            .any(|prepended| Rc::ptr_eq(prepended, module))
    }

    /// The mixin chain expanded through each module's own mixins, in method
    /// and constant lookup order. A module included into an already-included
    /// module shows up here without the outer class being touched.
    pub fn transitive_mixins(&self) -> Vec<Rc<Class>> {
        self.linear_mixins().to_vec()
    }

    /// The modules mixed in here, each followed by the modules it brings in,
    /// in lookup order. A module that one included later also brings in sits
    /// where that one brings it, since including a module skips each module
    /// the chain already holds.
    pub fn linear_mixins(&self) -> Rc<[Rc<Class>]> {
        let state = method_state();
        if let Some((worked_under, chain)) = self.linear_mixins_cache.borrow().as_ref()
            && *worked_under == state
        {
            return Rc::clone(chain);
        }
        let chain: Rc<[Rc<Class>]> = self.work_out_linear_mixins().into();
        *self.linear_mixins_cache.borrow_mut() = Some((state, Rc::clone(&chain)));
        chain
    }

    fn work_out_linear_mixins(&self) -> Vec<Rc<Class>> {
        let expansions: Vec<Vec<Rc<Class>>> = self
            .mixin_chain()
            .iter()
            .map(|module| module.module_chain())
            .collect();
        // The last expansion each module appears in, which is the include
        // that brought it in first.
        let mut last_seen: std::collections::HashMap<*const Class, usize> =
            std::collections::HashMap::new();
        for (at, expansion) in expansions.iter().enumerate() {
            for module in expansion {
                last_seen.insert(Rc::as_ptr(module), at);
            }
        }
        let mut placed: std::collections::HashSet<*const Class> = std::collections::HashSet::new();
        let mut chain: Vec<Rc<Class>> = Vec::new();
        for (at, expansion) in expansions.iter().enumerate() {
            for module in expansion {
                let held = Rc::as_ptr(module);
                if last_seen[&held] > at || !placed.insert(held) {
                    continue;
                }
                chain.push(Rc::clone(module));
            }
        }
        chain
    }

    /// A module's own ancestry: what is prepended to it, itself, and what it
    /// includes.
    fn module_chain(self: &Rc<Class>) -> Vec<Rc<Class>> {
        let mut chain = self.transitive_prepends();
        chain.push(Rc::clone(self));
        for module in self.linear_mixins().iter() {
            if !chain.iter().any(|held| Rc::ptr_eq(held, module)) {
                chain.push(Rc::clone(module));
            }
        }
        chain
    }

    /// Look up a method in this class's own table, without consulting mixins
    /// or the superclass chain.
    pub fn find_own_method(&self, name: &str) -> Option<Rc<Method>> {
        self.methods.borrow().get(name).map(Rc::clone)
    }

    /// A method written `def self.name` lives in the class's own table under
    /// a prefix rather than in the singleton class. Reading it back here lets
    /// the singleton class answer for it the way Ruby's does.
    pub fn find_attached_class_method(&self, name: &str) -> Option<Rc<Method>> {
        if !self.is_singleton_class() || name.starts_with("__class__") {
            return None;
        }
        match self.get_class_var("__attached__").view() {
            Some(crate::object::View::Class(attached))
            | Some(crate::object::View::Module(attached))
                if attached.holds_class_methods.get() =>
            {
                attached.find_own_method(&format!("__class__{}", name))
            }
            _ => None,
        }
    }

    /// Look up a method by walking the inheritance chain (own → mixins →
    /// superclass), answering from the cache while no method table or mixin
    /// chain has changed since the name was last looked up here.
    pub fn find_method(&self, name: &str) -> Option<Rc<Method>> {
        let state = method_state();
        if let Some((cached_under, found)) = self.method_cache.borrow().get(name)
            && *cached_under == state
        {
            return found.clone();
        }
        *self
            .method_walks
            .borrow_mut()
            .entry(name.to_string())
            .or_insert(0) += 1;
        let found = self.walk_for_method(name);
        // A lookup runs no program code, so the state it started under is the
        // one the answer holds for.
        self.method_cache
            .borrow_mut()
            .insert(name.to_string(), (state, found.clone()));
        found
    }

    /// How many times `find_method` walked the ancestry for `name` rather than
    /// answering from the cache.
    pub fn method_lookup_walks(&self, name: &str) -> u64 {
        self.method_walks.borrow().get(name).copied().unwrap_or(0)
    }

    fn walk_for_method(&self, name: &str) -> Option<Rc<Method>> {
        for prepended in self.prepends.borrow().iter() {
            if let Some(method) = prepended.find_method(name) {
                return Some(method);
            }
        }
        if let Some(method) = self.methods.borrow().get(name) {
            return Some(Rc::clone(method));
        }
        if let Some(method) = self.find_attached_class_method(name) {
            return Some(method);
        }

        if self.has_mixins() {
            for module in self.linear_mixins().iter() {
                if let Some(method) = module.find_own_method(name) {
                    return Some(method);
                }
            }
        }

        self.superclass
            .as_ref()
            .and_then(|superclass| superclass.find_method(name))
    }

    /// Look for a method along the superclass chain alone, skipping the
    /// modules mixed in along the way. A module's own methods are not
    /// inherited by whatever includes it, so a class method has to be found
    /// this way rather than through the full ancestry.
    pub fn find_inherited_method(&self, name: &str) -> Option<Rc<Method>> {
        if let Some(method) = self.methods.borrow().get(name) {
            return Some(Rc::clone(method));
        }
        self.superclass
            .as_ref()
            .and_then(|superclass| superclass.find_inherited_method(name))
    }

    /// Look up a method the same way `find_method` does, returning the module
    /// that actually defines it alongside the method itself.
    pub fn find_method_with_owner(self: &Rc<Class>, name: &str) -> Option<(Rc<Class>, Rc<Method>)> {
        for prepended in self.prepends.borrow().iter() {
            if let Some(found) = prepended.find_method_with_owner(name) {
                return Some(found);
            }
        }
        if let Some(method) = self.methods.borrow().get(name) {
            return Some((Rc::clone(self), Rc::clone(method)));
        }
        if let Some(method) = self.find_attached_class_method(name) {
            return Some((Rc::clone(self), method));
        }

        if self.has_mixins() {
            for module in self.linear_mixins().iter() {
                if let Some(method) = module.find_own_method(name) {
                    return Some((Rc::clone(module), method));
                }
            }
        }

        self.superclass
            .as_ref()
            .and_then(|superclass| superclass.find_method_with_owner(name))
    }

    /// True when this class object is a singleton class (created by
    /// `class << obj` or `obj.singleton_class`).
    pub fn is_singleton_class(&self) -> bool {
        self.get_class_var("__singleton__").is_some()
    }

    /// True when `other` appears in this module's ancestry (itself, its
    /// mixins, or any superclass and that superclass's mixins).
    pub fn has_ancestor(self: &Rc<Class>, other: &Rc<Class>) -> bool {
        if Rc::ptr_eq(self, other) {
            return true;
        }
        for prepended in self.prepends.borrow().iter() {
            if prepended.has_ancestor(other) {
                return true;
            }
        }
        for mixin in self.mixins.borrow().iter() {
            if mixin.has_ancestor(other) {
                return true;
            }
        }
        self.superclass
            .as_ref()
            .is_some_and(|superclass| superclass.has_ancestor(other))
    }

    /// Remove a method defined directly on this class.
    /// Returns true if the method was found and removed, false otherwise.
    pub fn remove_method(&self, name: &str) -> bool {
        method_named_changed(name);
        self.methods.borrow_mut().remove(name).is_some()
    }

    /// Create an alias for an existing method.
    /// Returns true if the source method was found and aliased, false otherwise.
    /// The alias inherits the original method's visibility — if `old_name` is
    /// private anywhere in the lookup chain, `new_name` is marked private on
    /// `self` too (matching MRI's behavior for `alias_method`).
    pub fn alias_method(self: &Rc<Class>, new_name: &str, old_name: &str) -> bool {
        if let Some(method) = self.find_method(old_name) {
            let mut aliased = (*method).clone();
            aliased.original_name = Some(
                method
                    .original_name
                    .clone()
                    .unwrap_or_else(|| method.name.clone()),
            );
            aliased.name = new_name.to_string();
            aliased.alias_origin = method
                .alias_origin
                .clone()
                .or_else(|| method.owner_class.clone());
            // The alias belongs to the class that made it, even when the
            // method it copies came from a prepended or included module.
            aliased.owner_class = Some(Rc::clone(self));
            aliased.owner = Some(self.ruby_name());
            method_named_changed(new_name);
            self.methods
                .borrow_mut()
                .insert(new_name.to_string(), std::rc::Rc::new(aliased));
            if self.is_method_protected_in_chain(old_name) {
                self.set_method_protected(new_name.to_string());
            } else if self.is_method_private_in_chain(old_name) {
                self.set_method_private(new_name.to_string());
            }
            return true;
        }
        // Singleton class of a class/module: `def self.x` methods live on
        // the attached class under the `__class__` prefix, not on the
        // singleton class itself. Alias from there so e.g. mspec's mock
        // installer can save a class method aside.
        if self.get_class_var("__singleton__").is_some()
            && let Some(View::Class(attached) | View::Module(attached)) =
                self.get_class_var("__attached__").view()
            && let Some(method) = attached.find_method(&format!("__class__{}", old_name))
        {
            method_state_changed();
            self.methods
                .borrow_mut()
                .insert(new_name.to_string(), method);
            return true;
        }
        false
    }

    /// Check whether a method is marked protected anywhere on this class or
    /// its ancestor chain. An explicit `public :name` override on a closer
    /// class shadows a more distant protected marking.
    fn is_method_protected_in_chain(&self, name: &str) -> bool {
        if self.has_public_override(name) {
            return false;
        }
        if self.is_method_protected(name) {
            return true;
        }
        for mixin in self.mixins.borrow().iter() {
            if mixin.has_public_override(name) {
                return false;
            }
            if mixin.is_method_protected(name) {
                return true;
            }
        }
        self.superclass
            .as_ref()
            .is_some_and(|sc| sc.is_method_protected_in_chain(name))
    }

    /// Check whether a method is marked private anywhere on this class or its
    /// ancestor chain (mixins + superclasses). An explicit `public :name`
    /// override on a closer class shadows a more distant private marking.
    fn is_method_private_in_chain(&self, name: &str) -> bool {
        if self.has_public_override(name) {
            return false;
        }
        if self.is_method_private(name) {
            return true;
        }
        for mixin in self.mixins.borrow().iter() {
            if mixin.has_public_override(name) {
                return false;
            }
            if mixin.is_method_private(name) {
                return true;
            }
        }
        if let Some(sc) = &self.superclass {
            return sc.is_method_private_in_chain(name);
        }
        false
    }

    /// Return a list of method names defined directly on this class.
    pub fn method_names(&self) -> Vec<String> {
        let mut names = self.methods.borrow().keys().cloned().collect::<Vec<_>>();
        // A singleton class answers for the `def self.name` methods its
        // attached class holds under a prefix.
        if self.is_singleton_class()
            && let attached_to = self.get_class_var("__attached__")
            && let Some(
                crate::object::View::Class(attached) | crate::object::View::Module(attached),
            ) = attached_to.view()
        {
            for held in attached.methods.borrow().keys() {
                if let Some(plain) = held.strip_prefix("__class__")
                    && !names.iter().any(|name| name == plain)
                {
                    names.push(plain.to_string());
                }
            }
        }
        names.sort();
        names
    }
}
