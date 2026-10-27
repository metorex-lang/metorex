// Scope and variable management for Metorex
// This module implements lexical scoping with scope chain traversal

use crate::object::Object;
use crate::object::View;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

thread_local! {
    static LAST_VERSION: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// A version no scope on this thread has had, which is what lets a scope
/// be told by its address and version from one that took its place.
fn next_version() -> u64 {
    LAST_VERSION.with(|last| {
        last.set(last.get() + 1);
        last.get()
    })
}

/// How many bindings an emptied scope keeps for `define_named`.
const SPARE_BINDINGS: usize = 8;

/// Represents a single scope in the scope chain
/// Each scope can have a parent scope, forming a chain for variable lookup
#[derive(Debug)]
pub struct Scope {
    /// Variable storage: maps variable names to shared mutable references
    /// This allows closures to mutate captured variables
    /// The names this scope binds, in the order they were first bound.
    variables: crate::fast_hash::NameIndexMap<String, Rc<RefCell<Object>>>,

    /// Reference to the parent scope (None for global scope)
    parent: Option<Rc<RefCell<Scope>>>,

    /// Whether this scope is a method boundary. A method body cannot see the
    /// caller's locals, so `local_variables` stops here.
    is_method_boundary: bool,

    /// Names carried in from a Binding, in the order that Binding reported
    /// them. Code run through a Binding names its own locals first and these
    /// after, which is the order Ruby reports them in.
    inherited_names: Vec<String>,

    /// Names bound without the program declaring them, which are not locals
    /// of this scope and so are left out of `local_variables`.
    hidden_names: HashSet<String>,

    /// Names bound ahead of the assignment that introduces them, the way
    /// Ruby's parser reserves a local for the rest of the body. A block
    /// written before that assignment does not close over one of these:
    /// Ruby's local does not exist yet where the block was written.
    hoisted_names: std::collections::HashSet<String, crate::fast_hash::NameHashing>,

    /// The names a block closed over where it was written, shared with every
    /// call of the block. They are read and written through, after this
    /// scope's own names and before its parent's, and are not locals of this
    /// scope.
    captured: Option<CapturedNames>,
    /// The locals listed where a block was written, in the order listed
    /// there, which `local_variables` lists in the block though the ones
    /// assigned after it cannot be read in it.
    reserved: Rc<Vec<String>>,
    /// The names and cells of bindings an earlier use of this scope let go
    /// of, which nothing else held, kept for `define_named` to bind again
    /// rather than allocate.
    spare_bindings: Vec<(String, Rc<RefCell<Object>>)>,
    /// A number that changes, to one no scope has had, each time the names
    /// this scope binds, reserves, or captures change, which a kept block
    /// compares.
    version: u64,
}

/// The names a block closes over, each with the cell it shares with the
/// scope it was written in.
pub type CapturedNames = Rc<Captures>;

/// One closed-over name and the cell it shares with its scope.
pub type Capture = (Rc<str>, Rc<RefCell<Object>>);

/// How many closed-over names a lookup reads through in order before it
/// reads them through an index instead.
const NAMES_FOUND_IN_ORDER: usize = 16;

/// Closed-over names in the order `local_variables` lists them, each with
/// the cell it shares with the scope it was written in.
#[derive(Debug, Default)]
pub struct Captures {
    entries: Box<[Capture]>,
    /// Where each name sits, made the first time a name is looked up among
    /// more than `NAMES_FOUND_IN_ORDER`.
    places: std::cell::OnceCell<Box<crate::fast_hash::NameMap<Rc<str>, u32>>>,
}

impl Captures {
    /// Names and cells with no name given twice.
    pub fn new(mut entries: Vec<Capture>) -> Self {
        crate::parser::tighten(&mut entries);
        Captures {
            entries: entries.into_boxed_slice(),
            places: std::cell::OnceCell::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Where `name` sits among the names.
    pub fn get_index_of(&self, name: &str) -> Option<usize> {
        if self.entries.len() <= NAMES_FOUND_IN_ORDER {
            return self.entries.iter().position(|(held, _)| &**held == name);
        }
        let places = self.places.get_or_init(|| {
            Box::new(
                self.entries
                    .iter()
                    .enumerate()
                    .map(|(at, (held, _))| (Rc::clone(held), at as u32))
                    .collect(),
            )
        });
        places.get(name).map(|&at| at as usize)
    }

    /// The name at `at` and its cell.
    pub fn get_index(&self, at: usize) -> Option<(&str, &Rc<RefCell<Object>>)> {
        self.entries.get(at).map(|(name, cell)| (&**name, cell))
    }

    pub fn get(&self, name: &str) -> Option<&Rc<RefCell<Object>>> {
        self.get_index_of(name).map(|at| &self.entries[at].1)
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.get_index_of(name).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Rc<RefCell<Object>>)> {
        self.entries.iter().map(|(name, cell)| (&**name, cell))
    }

    /// Bind `name` to `cell`, in its place when it is one of the names and
    /// after them when not.
    pub fn insert(&mut self, name: &str, cell: Rc<RefCell<Object>>) {
        if let Some(at) = self.get_index_of(name) {
            self.entries[at].1 = cell;
            return;
        }
        let mut entries = std::mem::take(&mut self.entries).into_vec();
        entries.push((crate::fast_hash::shared_name(name), cell));
        *self = Captures::new(entries);
    }
}

impl Clone for Captures {
    fn clone(&self) -> Self {
        Captures::new(self.entries.to_vec())
    }
}

impl PartialEq for Captures {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl Scope {
    /// The number the names this scope binds were last changed to.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// The scope this one reads a name from when it holds none of its own.
    pub fn parent(&self) -> Option<&Rc<RefCell<Scope>>> {
        self.parent.as_ref()
    }

    /// Creates a new scope with no parent (global scope)
    pub fn new() -> Self {
        Scope {
            variables: crate::fast_hash::NameIndexMap::default(),
            parent: None,
            is_method_boundary: false,
            inherited_names: Vec::new(),
            hidden_names: HashSet::new(),
            hoisted_names: Default::default(),
            captured: None,
            reserved: no_reserved_names(),
            spare_bindings: Vec::new(),
            version: next_version(),
        }
    }

    /// Creates a new scope with the given parent scope
    pub fn with_parent(parent: Rc<RefCell<Scope>>) -> Self {
        Scope {
            variables: crate::fast_hash::NameIndexMap::default(),
            parent: Some(parent),
            is_method_boundary: false,
            inherited_names: Vec::new(),
            hidden_names: HashSet::new(),
            hoisted_names: Default::default(),
            captured: None,
            reserved: no_reserved_names(),
            spare_bindings: Vec::new(),
            version: next_version(),
        }
    }

    /// Let go of everything this scope, which nothing holds any more,
    /// binds or reads through, so what its locals held can be freed while
    /// it waits to be used again. It takes a new version, so nothing
    /// compared against an earlier version takes it for the scope it was.
    pub fn empty(&mut self) {
        for (name, cell) in self.variables.drain(..) {
            if self.spare_bindings.len() < SPARE_BINDINGS
                && Rc::strong_count(&cell) == 1
                && Rc::weak_count(&cell) == 0
            {
                *cell.borrow_mut() = Object::Nil;
                self.spare_bindings.push((name, cell));
            }
        }
        self.parent = None;
        self.is_method_boundary = false;
        self.inherited_names.clear();
        self.hidden_names.clear();
        self.hoisted_names.clear();
        self.captured = None;
        self.reserved = no_reserved_names();
        self.version = next_version();
    }

    /// Put this emptied scope under `parent`.
    pub fn renew(&mut self, parent: Rc<RefCell<Scope>>) {
        self.parent = Some(parent);
        self.version = next_version();
    }

    /// Defines a new variable in the current scope
    /// If the variable already exists in this scope, it will be overwritten
    /// Drop `name` from the names hidden or hoisted here, without hashing it
    /// when there are none.
    fn forget_hidden_and_hoisted(&mut self, name: &str) {
        if !self.hidden_names.is_empty() {
            self.hidden_names.remove(name);
        }
        if !self.hoisted_names.is_empty() {
            self.hoisted_names.remove(name);
        }
    }

    /// `define`, taking a binding this scope let go of earlier, when it
    /// has one, rather than allocating the name and the cell.
    pub fn define_named(&mut self, name: &str, value: Object) {
        self.version = next_version();
        self.forget_hidden_and_hoisted(name);
        let (key, cell) = match self.spare_bindings.pop() {
            Some((mut key, cell)) => {
                key.clear();
                key.push_str(name);
                *cell.borrow_mut() = value;
                (key, cell)
            }
            None => (name.to_string(), Rc::new(RefCell::new(value))),
        };
        self.variables.insert(key, cell);
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.version = next_version();
        self.forget_hidden_and_hoisted(&name);
        self.variables.insert(name, Rc::new(RefCell::new(value)));
    }

    /// Defines a new variable in the current scope with a shared reference
    /// Used when a closure defines a captured variable
    pub fn define_shared(&mut self, name: String, value: Rc<RefCell<Object>>) {
        self.version = next_version();
        self.forget_hidden_and_hoisted(&name);
        self.variables.insert(name, value);
    }

    /// Binds a name carried in from a Binding. It behaves like any other
    /// local, and `local_variables` reports it after the scope's own.
    pub fn define_inherited(&mut self, name: String, value: Rc<RefCell<Object>>) {
        self.version = next_version();
        self.forget_hidden_and_hoisted(&name);
        if !self.inherited_names.contains(&name) {
            self.inherited_names.push(name.clone());
        }
        self.variables.insert(name, value);
    }

    /// Read and write through the names a block closed over, without
    /// copying them into this scope.
    pub fn attach_reserved(&mut self, reserved: Rc<Vec<String>>) {
        self.version = next_version();
        self.reserved = reserved;
    }

    pub fn attach_captured(&mut self, captured: CapturedNames) {
        self.version = next_version();
        self.captured = Some(captured);
    }

    /// The names the block this scope runs closed over, when it runs one.
    pub fn captured_names(&self) -> Option<&CapturedNames> {
        self.captured.as_ref()
    }

    /// Whether this scope binds `name` itself, leaving out what it closed
    /// over.
    pub fn binds(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// The cell a block closed over for `name`, when this scope runs one.
    fn captured_ref(&self, name: &str) -> Option<&Rc<RefCell<Object>>> {
        self.captured.as_ref().and_then(|held| held.get(name))
    }

    /// Binds a name the program did not declare, so `local_variables` and a
    /// Binding leave it out. The implicit `it` parameter is bound this way.
    pub fn define_hidden(&mut self, name: String, value: Object) {
        self.version = next_version();
        self.hoisted_names.remove(&name);
        self.hidden_names.insert(name.clone());
        self.variables.insert(name, Rc::new(RefCell::new(value)));
    }

    /// The implicit parameters this scope's block took, `it` or `_1` to
    /// `_9`, in the order they were bound, with their cells.
    pub fn implicit_parameters(&self) -> Vec<(String, Rc<RefCell<Object>>)> {
        self.ordered_names()
            .filter(|name| self.hidden_names.contains(*name))
            .filter(|name| name.as_str() == "it" || crate::parser::names_a_numbered_parameter(name))
            .filter_map(|name| {
                self.variables
                    .get(name)
                    .map(|cell| (name.clone(), Rc::clone(cell)))
            })
            .collect()
    }

    /// Reserves `name` for the rest of this scope, the way Ruby's parser
    /// does at the assignment that introduces a local. Reading it answers
    /// nil, and a block written before the assignment does not close over it.
    pub fn hoist(&mut self, name: String) {
        self.version = next_version();
        self.hoisted_names.insert(name.clone());
        self.variables
            .insert(name, Rc::new(RefCell::new(Object::Nil)));
    }

    /// Say that the assignment `name` was reserved for has begun, so code
    /// written in its value closes over the name.
    pub fn unhoist(&mut self, name: &str) {
        if self.hoisted_names.remove(name) {
            self.version = next_version();
            return;
        }
        if let Some(parent) = &self.parent
            && !self.is_method_boundary
        {
            parent.borrow_mut().unhoist(name);
        }
    }

    /// Whether `name` is only reserved for a later assignment and has not
    /// been assigned yet. Ruby's parser introduces a local at the assignment
    /// that writes it, so a block written above that line does not see one.
    pub fn is_only_hoisted(&self, name: &str) -> bool {
        if self.variables.contains_key(name) {
            return self.hoisted_names.contains(name);
        }
        if self.captured_ref(name).is_some() {
            return false;
        }
        match &self.parent {
            Some(parent) => parent.borrow().is_only_hoisted(name),
            None => false,
        }
    }

    /// Remove a variable from the current scope only (does not walk parents).
    pub fn undefine(&mut self, name: &str) {
        self.version = next_version();
        self.variables.shift_remove(name);
    }

    /// This scope's names in the order they were first bound.
    fn ordered_names(&self) -> impl Iterator<Item = &String> {
        self.variables.keys()
    }

    /// Gets a variable value by traversing the scope chain
    /// Returns None if the variable is not found in any scope
    pub fn get(&self, name: &str) -> Option<Object> {
        // First, check if the variable exists in this scope
        if let Some(value_ref) = self.variables.get(name) {
            return Some(value_ref.borrow().clone());
        }
        if let Some(value_ref) = self.captured_ref(name) {
            return Some(value_ref.borrow().clone());
        }

        // If not found, check the parent scope recursively
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }

        // Variable not found in any scope
        None
    }

    /// Gets a shared reference to a variable by traversing the scope chain
    /// Used for closure capture to enable mutable closures
    pub fn get_ref(&self, name: &str) -> Option<Rc<RefCell<Object>>> {
        // First, check if the variable exists in this scope
        if let Some(value_ref) = self.variables.get(name) {
            return Some(value_ref.clone());
        }
        if let Some(value_ref) = self.captured_ref(name) {
            return Some(value_ref.clone());
        }

        // If not found, check the parent scope recursively
        if let Some(parent) = &self.parent {
            return parent.borrow().get_ref(name);
        }

        // Variable not found in any scope
        None
    }

    /// Sets a variable value by traversing the scope chain
    /// Returns true if the variable was found and updated, false otherwise
    /// This method will NOT create a new variable if it doesn't exist
    pub fn set(&mut self, name: &str, value: Object) -> bool {
        // First, check if the variable exists in this scope
        if let Some(value_ref) = self.variables.get(name) {
            note_callable_change(&value_ref.borrow(), &value);
            *value_ref.borrow_mut() = value;
            // The assignment the parser reserved the name for has now run,
            // so a block written from here on closes over it.
            if self.hoisted_names.remove(name) {
                self.version = next_version();
            }
            return true;
        }
        if let Some(value_ref) = self.captured_ref(name) {
            note_callable_change(&value_ref.borrow(), &value);
            *value_ref.borrow_mut() = value;
            return true;
        }

        // A method or block scope does not assign to a name it never bound:
        // Ruby makes that a new local of its own. Only the names it captured
        // reach outward, and those are already bound here as shared cells.
        if self.is_method_boundary {
            return false;
        }

        // If not found, try to set it in the parent scope
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().set(name, value);
        }

        // Variable not found in any scope
        false
    }

    /// Gets a variable at a specific depth in the scope chain
    /// depth=0 means current scope, depth=1 means parent, etc.
    /// This is useful for closure resolution where we know the exact depth
    pub fn get_at(&self, depth: usize, name: &str) -> Option<Object> {
        if depth == 0 {
            return self
                .variables
                .get(name)
                .or_else(|| self.captured_ref(name))
                .map(|v| v.borrow().clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get_at(depth - 1, name);
        }

        None
    }

    /// Sets a variable at a specific depth in the scope chain
    /// depth=0 means current scope, depth=1 means parent, etc.
    /// Returns true if successful, false if the depth is invalid or variable doesn't exist
    pub fn set_at(&mut self, depth: usize, name: &str, value: Object) -> bool {
        if depth == 0 {
            if let Some(value_ref) = self.variables.get(name).or_else(|| self.captured_ref(name)) {
                note_callable_change(&value_ref.borrow(), &value);
                *value_ref.borrow_mut() = value;
                return true;
            }
            return false;
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().set_at(depth - 1, name, value);
        }

        false
    }

    /// Collects all variables from the entire scope chain
    /// Returns a HashMap with all visible variables (parent scope vars may be shadowed)
    pub fn collect_all_vars(&self) -> HashMap<String, Object> {
        let mut all_vars = HashMap::new();

        // Start from parent and work backwards, so that closer scopes override farther ones
        if let Some(parent) = &self.parent {
            all_vars = parent.borrow().collect_all_vars();
        }
        if let Some(captured) = &self.captured {
            for (name, value_ref) in captured.iter() {
                all_vars.insert(name.to_string(), value_ref.borrow().clone());
            }
        }

        // Now add this scope's variables (potentially overriding parent values),
        // in the order they were bound, which is the order a block written
        // here reports them in.
        let bound_in_order = self.ordered_names();
        for name in bound_in_order {
            let value_ref = &self.variables[name];
            all_vars.insert(name.clone(), value_ref.borrow().clone());
        }

        all_vars
    }

    /// Names bound in this scope alone, excluding those a block captured.
    pub fn own_variable_names(&self) -> Vec<String> {
        self.ordered_names()
            .filter(|name| !self.hidden_names.contains(*name))
            .cloned()
            .collect()
    }

    /// Names bound in this scope and the enclosing scopes a Ruby local would
    /// be visible from. The walk stops at the root, which holds the builtins,
    /// and at a method boundary, whose locals belong to the method rather
    /// than to the block running inside it.
    pub fn collect_local_variable_names(&self) -> Vec<String> {
        // A scope's own locals come before the ones carried in from a
        // Binding, which is the order Ruby reports them in.
        let mut names: Vec<String> = self
            .own_variable_names()
            .into_iter()
            .filter(|name| !self.inherited_names.contains(name))
            .collect();
        names.extend(
            self.inherited_names
                .iter()
                .filter(|name| self.variables.contains_key(*name))
                .filter(|name| !self.hidden_names.contains(*name))
                .cloned(),
        );
        // A method's own locals end the chain: what encloses the method is
        // not in scope inside it. A block, on the other hand, closes over the
        // scope it was written in, so the walk carries on through it.
        if self.is_method_boundary {
            return names;
        }
        if let Some(parent) = &self.parent {
            let parent_ref = parent.borrow();
            if parent_ref.parent.is_some() {
                names.extend(parent_ref.collect_local_variable_names());
            }
        }
        names
    }

    /// The names a Binding holds. A block closes over the locals of the scope
    /// it was written in, and reading or writing one of those through a
    /// binding reaches the same reference, so a captured name counts here
    /// even though it is not the block's own.
    pub fn collect_binding_variable_names(&self) -> Vec<String> {
        let holds_a_local = |name: &String| {
            !self.hidden_names.contains(name)
                && self
                    .variables
                    .get(name)
                    .is_some_and(|value| !names_a_definition_under(name, &value.borrow()))
        };
        // A scope's own locals come before the ones it can see through the
        // scopes enclosing it, which is the order Ruby reports them in.
        let mut names: Vec<String> = self
            .ordered_names()
            .filter(|name| !self.inherited_names.contains(*name))
            .filter(|name| holds_a_local(name))
            .cloned()
            .collect();
        names.extend(
            self.inherited_names
                .iter()
                .filter(|name| self.variables.contains_key(*name))
                .filter(|name| holds_a_local(name))
                .cloned(),
        );
        // A block lists the locals of the scope it was written in, in the
        // order they were listed there, including the ones assigned after it.
        for name in self.reserved.iter() {
            if !names.contains(name) {
                names.push(name.clone());
            }
        }
        if !self.is_method_boundary
            && let Some(parent) = &self.parent
        {
            let parent_ref = parent.borrow();
            if parent_ref.parent.is_some() {
                names.extend(parent_ref.collect_binding_variable_names());
            }
        }
        // A name a block captured is a local of the scope the block was
        // written in. That scope is normally the one the walk above reaches,
        // but a block outliving it keeps the name and nothing else does.
        let mut closed_over: Vec<String> = Vec::new();
        if let Some(captured) = &self.captured {
            for (name, value) in captured.iter() {
                if !self.variables.contains_key(name)
                    && !self.hidden_names.contains(name)
                    && !names_a_definition_under(name, &value.borrow())
                    && !names.iter().any(|listed| listed == name)
                    && !closed_over.iter().any(|listed| listed == name)
                {
                    closed_over.push(name.to_string());
                }
            }
        }
        names.extend(closed_over);
        names
    }

    /// Mark this scope as a method boundary.
    pub fn mark_method_boundary(&mut self) {
        self.version = next_version();
        self.is_method_boundary = true;
    }

    /// The reference this scope holds for `name`, ignoring enclosing scopes.
    pub fn own_var_ref(&self, name: &str) -> Option<Rc<RefCell<Object>>> {
        self.variables
            .get(name)
            .or_else(|| self.captured_ref(name))
            .cloned()
    }

    /// Whether this scope binds or captured `name` itself.
    pub fn holds_own(&self, name: &str) -> bool {
        self.variables.contains_key(name) || self.captured_ref(name).is_some()
    }

    /// The value this scope binds `name` to itself, without reading through
    /// what it captured or its parent.
    pub fn own_value(&self, name: &str) -> Option<Object> {
        self.variables.get(name).map(|held| held.borrow().clone())
    }

    /// Whether this scope is the root of the chain.
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// The cells of every name the scope chain shows, closer scopes first,
    /// for a block written here to close over.
    pub fn collect_all_var_refs(&self) -> Captures {
        let mut gathered = Vec::new();
        let mut seen = crate::fast_hash::NameSet::default();
        self.gather_var_refs(&mut gathered, &mut seen);
        Captures::new(gathered)
    }

    fn gather_var_refs(
        &self,
        gathered: &mut Vec<Capture>,
        seen: &mut crate::fast_hash::NameSet<Rc<str>>,
    ) {
        let mut take = |name: &str, cell: &Rc<RefCell<Object>>| {
            if seen.contains(name) {
                return;
            }
            let shared = crate::fast_hash::shared_name(name);
            seen.insert(Rc::clone(&shared));
            gathered.push((shared, Rc::clone(cell)));
        };
        // This scope's own variables come first, in the order they were
        // bound, then what it closed over, then what encloses it. A name
        // already taken stays with the closer scope, and the order is the
        // order `local_variables` reports them in.
        for name in self.ordered_names() {
            // A name the program never declared is not one a block closes
            // over: a block reading `it` takes the argument it was handed
            // rather than the `it` of the block around it.
            if self.hoisted_names.contains(name) || self.hidden_names.contains(name) {
                continue;
            }
            take(name, &self.variables[name]);
        }
        if let Some(captured) = &self.captured {
            for (name, cell) in captured.iter() {
                take(name, cell);
            }
        }
        // A method body ends the walk: the scope above it holds the program's
        // own top-level locals, which a block written inside a method does not
        // close over.
        if !self.is_method_boundary
            && let Some(parent) = &self.parent
        {
            parent.borrow().gather_var_refs(gathered, seen);
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

/// Whether the value `name` holds is a definition rather than a local: a
/// method, or a class or module bound under the constant a `class` or
/// `module` statement names. A lowercase local may hold a class as any
/// other value.
pub fn names_a_definition_under(name: &str, value: &Object) -> bool {
    match value.view() {
        View::Class(_) | View::Module(_) => {
            name.starts_with(|first: char| !first.is_lowercase() && first != '_')
        }
        _ => names_a_definition(value),
    }
}

/// Whether a binding holds a definition rather than a local: a method, a
/// class, or a module reached by name is not one of a scope's locals.
pub fn names_a_definition(value: &Object) -> bool {
    matches!(
        value.view(),
        View::Method(_) | View::Class(_) | View::Module(_) | View::NativeFunction(_)
    )
}

/// A local that held or now holds a method or a function changes which
/// names a block written in its scope lists as locals.
fn note_callable_change(held: &Object, value: &Object) {
    let callable = |object: &Object| {
        matches!(
            object.view(),
            crate::object::View::Method(_) | crate::object::View::NativeFunction(_)
        )
    };
    if callable(held) || callable(value) {
        crate::vm::block_reuse::scope_state_changed();
    }
}

thread_local! {
    static NO_RESERVED_NAMES: Rc<Vec<String>> = Rc::new(Vec::new());
}

/// The reserved names of a scope no block runs in, shared by all of them.
fn no_reserved_names() -> Rc<Vec<String>> {
    NO_RESERVED_NAMES.with(Rc::clone)
}
