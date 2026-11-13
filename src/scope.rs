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

/// The name a scope binds the receiver under.
const SELF: &str = "self";

/// How many bindings an emptied scope keeps for `define_named`.
const SPARE_BINDINGS: usize = 64;

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
    /// Hoisted names an earlier use of this scope let go of, kept for
    /// `hoist` to use again rather than allocate.
    spare_names: Vec<String>,
    /// A number that changes, to one no scope has had, each time the names
    /// this scope binds, reserves, or captures change, which a kept block
    /// compares.
    version: u64,
    /// The locals of the method body running in this scope, held by their
    /// place in the body's list rather than by name.
    frame: Option<Rc<FrameLayout>>,
    /// What this scope holds for each of the frame's locals.
    frame_locals: RefCell<Vec<FrameLocal>>,
    /// How many of `variables` were bound before the frame's locals, which
    /// `local_variables` lists ahead of them.
    bound_before_frame: usize,
    /// Cells of frame locals an earlier use of this scope let go of, which
    /// nothing else held, kept to bind frame locals again rather than
    /// allocate.
    spare_cells: Vec<Rc<RefCell<Object>>>,
    /// What this scope binds `self` to, which it holds apart from the
    /// names in `variables` and lists ahead of them.
    self_cell: Option<Rc<RefCell<Object>>>,
}

/// A method's parameters, in the order they were written, and then the
/// locals its body assigns, with where its unit's slots stand among them.
#[derive(Debug)]
pub struct FrameLayout {
    names: Box<[Rc<str>]>,
    /// The place of the local in each of the unit's slots, or `NO_PLACE`
    /// for one the scope binds by name.
    place_of_slot: Box<[u32]>,
    places: std::cell::OnceCell<crate::fast_hash::NameMap<Rc<str>, u32>>,
    /// Whether the method's positional parameters stand first, in order,
    /// once a binding of them has looked.
    parameters_lead: std::cell::OnceCell<bool>,
}

/// The place of a slot whose local the scope binds by name.
const NO_PLACE: u32 = u32::MAX;

impl FrameLayout {
    /// The layout of `names`, where `slots` names the local in each of the
    /// unit's slots that is one of them.
    pub fn new(names: Vec<String>, slots: &[Option<&str>]) -> Self {
        let place_of_slot = slots
            .iter()
            .map(|slot| {
                slot.and_then(|name| names.iter().position(|held| held == name))
                    .map_or(NO_PLACE, |place| place as u32)
            })
            .collect();
        FrameLayout {
            names: names.iter().map(|name| Rc::from(name.as_str())).collect(),
            place_of_slot,
            places: std::cell::OnceCell::new(),
            parameters_lead: std::cell::OnceCell::new(),
        }
    }

    /// The place of the local in the unit's `slot`.
    pub fn place_for_slot(&self, slot: usize) -> Option<usize> {
        match self.place_of_slot.get(slot) {
            Some(&place) if place != NO_PLACE => Some(place as usize),
            _ => None,
        }
    }

    /// Where `name` sits among the locals.
    fn place_of(&self, name: &str) -> Option<usize> {
        if self.names.len() <= NAMES_FOUND_IN_ORDER {
            return self.names.iter().position(|held| &**held == name);
        }
        let places = self.places.get_or_init(|| {
            self.names
                .iter()
                .enumerate()
                .map(|(at, held)| (Rc::clone(held), at as u32))
                .collect()
        });
        places.get(name).map(|&at| at as usize)
    }
}

/// What a scope holds for one of its frame's locals.
#[derive(Debug, Clone)]
enum FrameLocal {
    /// Nothing: the name is bound by name, or not at all.
    Unheld,
    /// Reserved for the assignment further down, reading nil, with the cell
    /// it reads from once something asked for one.
    Reserved(Option<Rc<RefCell<Object>>>),
    Bound(Rc<RefCell<Object>>),
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

    /// Hold `layout`'s locals by place, each reserved for its assignment,
    /// leaving out any this scope binds by name already when `bound_by_name`
    /// says it may bind one. A scope holding a frame keeps it, and answers
    /// false.
    pub fn attach_frame(&mut self, layout: &Rc<FrameLayout>, bound_by_name: bool) -> bool {
        if self.frame.is_some() {
            return false;
        }
        let locals = self.frame_locals.get_mut();
        locals.clear();
        for name in layout.names.iter() {
            locals.push(
                if !(bound_by_name && self.variables.contains_key(&**name)) {
                    FrameLocal::Reserved(None)
                } else {
                    FrameLocal::Unheld
                },
            );
        }
        self.bound_before_frame = self.variables.len();
        self.frame = Some(Rc::clone(layout));
        self.version = next_version();
        true
    }

    /// The cell this scope binds `self` in, when `name` is `self` and the
    /// scope binds it.
    fn self_ref(&self, name: &str) -> Option<&Rc<RefCell<Object>>> {
        if name == SELF {
            self.self_cell.as_ref()
        } else {
            None
        }
    }

    /// Whether this scope holds `layout`'s locals by place.
    pub fn holds_frame(&self, layout: &Rc<FrameLayout>) -> bool {
        self.frame
            .as_ref()
            .is_some_and(|held| Rc::ptr_eq(held, layout))
    }

    /// The cell of the frame local at `place`, binding one holding nil when
    /// it is reserved, or None when the scope does not hold it by place.
    pub fn frame_cell(&self, place: usize) -> Option<Rc<RefCell<Object>>> {
        let mut locals = self.frame_locals.borrow_mut();
        match locals.get(place)? {
            FrameLocal::Unheld => None,
            FrameLocal::Bound(cell) | FrameLocal::Reserved(Some(cell)) => Some(Rc::clone(cell)),
            FrameLocal::Reserved(None) => {
                let cell = Rc::new(RefCell::new(Object::Nil));
                locals[place] = FrameLocal::Reserved(Some(Rc::clone(&cell)));
                Some(cell)
            }
        }
    }

    /// Write `value` to the frame local at `place`, which ends its
    /// reservation, answering its cell, or None when the scope does not
    /// hold it by place.
    pub fn assign_frame_local(
        &mut self,
        place: usize,
        value: Object,
    ) -> Option<Rc<RefCell<Object>>> {
        let local = self.frame_locals.get_mut().get_mut(place)?;
        let cell = match local {
            FrameLocal::Unheld => return None,
            FrameLocal::Bound(cell) => {
                note_callable_change(&cell.borrow(), &value);
                *cell.borrow_mut() = value;
                return Some(Rc::clone(cell));
            }
            FrameLocal::Reserved(Some(cell)) => Rc::clone(cell),
            FrameLocal::Reserved(None) => self
                .spare_cells
                .pop()
                .unwrap_or_else(|| Rc::new(RefCell::new(Object::Nil))),
        };
        note_callable_change(&Object::Nil, &value);
        *cell.borrow_mut() = value;
        *local = FrameLocal::Bound(Rc::clone(&cell));
        self.version = next_version();
        Some(cell)
    }

    /// Whether the frame this scope holds starts with `parameters`, each
    /// named once, in the order given.
    pub fn frame_leads_with(&self, parameters: &[String]) -> bool {
        let Some(layout) = &self.frame else {
            return false;
        };
        *layout.parameters_lead.get_or_init(|| {
            parameters.len() <= layout.names.len()
                && parameters.iter().zip(layout.names.iter()).enumerate().all(
                    |(at, (parameter, held))| {
                        **held == **parameter && !parameters[..at].contains(parameter)
                    },
                )
        })
    }

    /// Write `value` to the frame local at `place`, which `frame_leads_with`
    /// said holds the parameter, answering the value back when the scope
    /// does not hold it by place.
    pub fn bind_frame_parameter(&mut self, place: usize, value: Object) -> Result<(), Object> {
        if matches!(self.frame_locals.get_mut()[place], FrameLocal::Unheld) {
            return Err(value);
        }
        self.assign_frame_local(place, value);
        Ok(())
    }

    /// Write `value` to the frame local at `place` when `name` is the local
    /// there, keeping the cell it is bound to, answering the value back when
    /// it is not.
    pub fn bind_frame_place(
        &mut self,
        place: usize,
        name: &str,
        value: Object,
    ) -> Result<(), Object> {
        let named = self
            .frame
            .as_ref()
            .and_then(|layout| layout.names.get(place))
            .is_some_and(|held| &**held == name);
        if !named || matches!(self.frame_locals.get_mut()[place], FrameLocal::Unheld) {
            return Err(value);
        }
        self.assign_frame_local(place, value);
        Ok(())
    }

    /// Where `name` is among the frame locals this scope holds.
    fn frame_place(&self, name: &str) -> Option<usize> {
        let place = self.frame.as_ref()?.place_of(name)?;
        let held = !matches!(self.frame_locals.borrow()[place], FrameLocal::Unheld);
        held.then_some(place)
    }

    /// The value of the frame local `name`, nil while it is reserved.
    fn frame_value(&self, name: &str) -> Option<Object> {
        let place = self.frame_place(name)?;
        match &self.frame_locals.borrow()[place] {
            FrameLocal::Bound(cell) | FrameLocal::Reserved(Some(cell)) => {
                Some(cell.borrow().clone())
            }
            _ => Some(Object::Nil),
        }
    }

    /// The cell of the frame local `name`, bound when it is reserved.
    fn frame_ref(&self, name: &str) -> Option<Rc<RefCell<Object>>> {
        self.frame_cell(self.frame_place(name)?)
    }

    /// Bind the frame local `name` to `cell`, answering the cell back when
    /// the scope does not hold `name` by place.
    fn bind_in_frame(
        &mut self,
        name: &str,
        cell: Rc<RefCell<Object>>,
    ) -> Option<Rc<RefCell<Object>>> {
        if name == SELF {
            self.version = next_version();
            self.self_cell = Some(cell);
            return None;
        }
        match self.frame_place(name) {
            Some(place) => {
                self.version = next_version();
                self.frame_locals.get_mut()[place] = FrameLocal::Bound(cell);
                None
            }
            None => Some(cell),
        }
    }

    /// Visit the names this scope binds in the order they were first bound,
    /// each with its cell, or None for a frame local still reserved, and
    /// whether it is reserved for an assignment further down.
    fn visit_in_order(&self, mut visit: impl FnMut(&str, Option<&Rc<RefCell<Object>>>, bool)) {
        if let Some(cell) = &self.self_cell {
            visit(SELF, Some(cell), false);
        }
        let before = self.bound_before_frame.min(self.variables.len());
        let (ahead, after) = self.variables.as_slice().split_at(before);
        for (name, cell) in ahead {
            visit(name, Some(cell), self.hoisted_names.contains(name.as_str()));
        }
        if let Some(layout) = &self.frame {
            for (name, local) in layout.names.iter().zip(self.frame_locals.borrow().iter()) {
                match local {
                    FrameLocal::Unheld => {}
                    FrameLocal::Reserved(cell) => visit(name, cell.as_ref(), true),
                    FrameLocal::Bound(cell) => visit(name, Some(cell), false),
                }
            }
        }
        for (name, cell) in after {
            visit(name, Some(cell), self.hoisted_names.contains(name.as_str()));
        }
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
            spare_names: Vec::new(),
            version: next_version(),
            frame: None,
            frame_locals: RefCell::new(Vec::new()),
            bound_before_frame: 0,
            spare_cells: Vec::new(),
            self_cell: None,
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
            spare_names: Vec::new(),
            version: next_version(),
            frame: None,
            frame_locals: RefCell::new(Vec::new()),
            bound_before_frame: 0,
            spare_cells: Vec::new(),
            self_cell: None,
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
        if !self.hidden_names.is_empty() {
            self.hidden_names.clear();
        }
        if !self.hoisted_names.is_empty() {
            for name in self.hoisted_names.drain() {
                if self.spare_names.len() < SPARE_BINDINGS {
                    self.spare_names.push(name);
                }
            }
        }
        self.captured = None;
        if !self.reserved.is_empty() {
            self.reserved = no_reserved_names();
        }
        if let Some(cell) = self.self_cell.take()
            && self.spare_cells.len() < SPARE_BINDINGS
            && Rc::strong_count(&cell) == 1
            && Rc::weak_count(&cell) == 0
        {
            *cell.borrow_mut() = Object::Nil;
            self.spare_cells.push(cell);
        }
        if self.frame.take().is_some() {
            for local in self.frame_locals.get_mut().drain(..) {
                if let FrameLocal::Bound(cell) | FrameLocal::Reserved(Some(cell)) = local
                    && self.spare_cells.len() < SPARE_BINDINGS
                    && Rc::strong_count(&cell) == 1
                    && Rc::weak_count(&cell) == 0
                {
                    *cell.borrow_mut() = Object::Nil;
                    self.spare_cells.push(cell);
                }
            }
            self.bound_before_frame = 0;
        }
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
            let taken = self.hoisted_names.take(name);
            self.keep_spare_name(taken);
        }
    }

    /// `define`, taking a binding this scope let go of earlier, when it
    /// has one, rather than allocating the name and the cell.
    pub fn define_named(&mut self, name: &str, value: Object) {
        self.version = next_version();
        if name == SELF {
            let cell = match self.spare_cells.pop() {
                Some(cell) => {
                    *cell.borrow_mut() = value;
                    cell
                }
                None => Rc::new(RefCell::new(value)),
            };
            self.self_cell = Some(cell);
            return;
        }
        if self.frame.is_some() && self.frame_place(name).is_some() {
            self.bind_in_frame(name, Rc::new(RefCell::new(value)));
            return;
        }
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
        self.define_shared(name, Rc::new(RefCell::new(value)));
    }

    /// Defines a new variable in the current scope with a shared reference
    /// Used when a closure defines a captured variable
    pub fn define_shared(&mut self, name: String, value: Rc<RefCell<Object>>) {
        self.version = next_version();
        let Some(value) = self.bind_in_frame(&name, value) else {
            return;
        };
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
        let Some(value) = self.bind_in_frame(&name, value) else {
            return;
        };
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
        self.self_ref(name).is_some()
            || self.variables.contains_key(name)
            || self.frame_place(name).is_some()
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
        let cell = Rc::new(RefCell::new(value));
        let Some(cell) = self.bind_in_frame(&name, cell) else {
            return;
        };
        self.variables.insert(name, cell);
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
    pub fn hoist(&mut self, name: &str) {
        self.version = next_version();
        if let Some(place) = self.frame_place(name) {
            self.frame_locals.get_mut()[place] = FrameLocal::Reserved(None);
            return;
        }
        if !self.hoisted_names.contains(name) {
            let mut held = self.spare_names.pop().unwrap_or_default();
            held.clear();
            held.push_str(name);
            self.hoisted_names.insert(held);
        }
        let (key, cell) = match self.spare_bindings.pop() {
            Some((mut key, cell)) => {
                key.clear();
                key.push_str(name);
                *cell.borrow_mut() = Object::Nil;
                (key, cell)
            }
            None => (name.to_string(), Rc::new(RefCell::new(Object::Nil))),
        };
        self.variables.insert(key, cell);
    }

    /// Keep a name this scope stopped hoisting for `hoist` to use again.
    fn keep_spare_name(&mut self, name: Option<String>) {
        if let Some(name) = name
            && self.spare_names.len() < SPARE_BINDINGS
        {
            self.spare_names.push(name);
        }
    }

    /// Say that the assignment `name` was reserved for has begun, so code
    /// written in its value closes over the name.
    pub fn unhoist(&mut self, name: &str) {
        if let Some(place) = self.frame_place(name) {
            self.unhoist_frame_local(place);
            return;
        }
        if let Some(taken) = self.hoisted_names.take(name) {
            self.keep_spare_name(Some(taken));
            self.version = next_version();
            return;
        }
        if let Some(parent) = &self.parent
            && !self.is_method_boundary
        {
            parent.borrow_mut().unhoist(name);
        }
    }

    /// `unhoist` for the frame local at `place`, answering whether the
    /// scope holds it by place.
    pub fn unhoist_frame_local(&mut self, place: usize) -> bool {
        let Some(local) = self.frame_locals.get_mut().get_mut(place) else {
            return false;
        };
        match local {
            FrameLocal::Unheld => false,
            FrameLocal::Bound(_) => true,
            FrameLocal::Reserved(cell) => {
                let cell = cell.take().unwrap_or_else(|| {
                    self.spare_cells
                        .pop()
                        .unwrap_or_else(|| Rc::new(RefCell::new(Object::Nil)))
                });
                *local = FrameLocal::Bound(cell);
                self.version = next_version();
                true
            }
        }
    }

    /// Whether `name` is only reserved for a later assignment and has not
    /// been assigned yet. Ruby's parser introduces a local at the assignment
    /// that writes it, so a block written above that line does not see one.
    pub fn is_only_hoisted(&self, name: &str) -> bool {
        if self.self_ref(name).is_some() {
            return false;
        }
        if let Some(place) = self.frame_place(name) {
            return matches!(self.frame_locals.borrow()[place], FrameLocal::Reserved(_));
        }
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
        if name == SELF {
            self.self_cell = None;
            return;
        }
        if let Some(place) = self.frame_place(name) {
            self.frame_locals.get_mut()[place] = FrameLocal::Unheld;
            return;
        }
        if let Some(at) = self.variables.get_index_of(name) {
            self.variables.shift_remove_index(at);
            if at < self.bound_before_frame {
                self.bound_before_frame -= 1;
            }
        }
    }

    /// This scope's names in the order they were first bound.
    fn ordered_names(&self) -> impl Iterator<Item = &String> {
        self.variables.keys()
    }

    /// Gets a variable value by traversing the scope chain
    /// Returns None if the variable is not found in any scope
    pub fn get(&self, name: &str) -> Option<Object> {
        if let Some(value) = self.get_here(name) {
            return Some(value);
        }
        self.parent.as_ref()?.borrow().get(name)
    }

    /// `get`, taking the root scope to bind no `name` while its address and
    /// version are what `root_lacks` holds, and noting them there when it
    /// finds that it binds none.
    pub fn get_through(
        &self,
        name: &str,
        root_lacks: &std::cell::Cell<(usize, u64)>,
    ) -> Option<Object> {
        let Some(parent) = &self.parent else {
            let root = (std::ptr::from_ref(self) as usize, self.version);
            if root_lacks.get() == root {
                return None;
            }
            let found = self.get_here(name);
            if found.is_none() {
                root_lacks.set(root);
            }
            return found;
        };
        if let Some(value) = self.get_here(name) {
            return Some(value);
        }
        parent.borrow().get_through(name, root_lacks)
    }

    /// What this scope itself binds `name` to, or what it closed over.
    fn get_here(&self, name: &str) -> Option<Object> {
        if let Some(cell) = self.self_ref(name) {
            return Some(cell.borrow().clone());
        }
        // First, check if the variable exists in this scope
        if let Some(value_ref) = self.variables.get(name) {
            return Some(value_ref.borrow().clone());
        }
        if self.frame.is_some()
            && let Some(value) = self.frame_value(name)
        {
            return Some(value);
        }
        if let Some(value_ref) = self.captured_ref(name) {
            return Some(value_ref.borrow().clone());
        }
        None
    }

    /// Gets a shared reference to a variable by traversing the scope chain
    /// Used for closure capture to enable mutable closures
    pub fn get_ref(&self, name: &str) -> Option<Rc<RefCell<Object>>> {
        if let Some(cell) = self.self_ref(name) {
            return Some(Rc::clone(cell));
        }
        // First, check if the variable exists in this scope
        if let Some(value_ref) = self.variables.get(name) {
            return Some(value_ref.clone());
        }
        if self.frame.is_some()
            && let Some(cell) = self.frame_ref(name)
        {
            return Some(cell);
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
        if let Some(cell) = self.self_ref(name) {
            *cell.borrow_mut() = value;
            return true;
        }
        // First, check if the variable exists in this scope
        if let Some(value_ref) = self.variables.get(name) {
            note_callable_change(&value_ref.borrow(), &value);
            *value_ref.borrow_mut() = value;
            // The assignment the parser reserved the name for has now run,
            // so a block written from here on closes over it.
            if !self.hoisted_names.is_empty()
                && let Some(taken) = self.hoisted_names.take(name)
            {
                self.keep_spare_name(Some(taken));
                self.version = next_version();
            }
            return true;
        }
        if let Some(place) = self.frame_place(name) {
            self.assign_frame_local(place, value);
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
            if let Some(cell) = self.self_ref(name) {
                return Some(cell.borrow().clone());
            }
            return self
                .variables
                .get(name)
                .map(|v| v.borrow().clone())
                .or_else(|| self.frame_value(name))
                .or_else(|| self.captured_ref(name).map(|v| v.borrow().clone()));
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
            if let Some(value_ref) = self
                .self_ref(name)
                .cloned()
                .or_else(|| self.variables.get(name).cloned())
                .or_else(|| self.frame_ref(name))
                .or_else(|| self.captured_ref(name).cloned())
            {
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
        self.visit_in_order(|name, cell, _| {
            let value = cell.map_or(Object::Nil, |cell| cell.borrow().clone());
            all_vars.insert(name.to_string(), value);
        });

        all_vars
    }

    /// Names bound in this scope alone, excluding those a block captured.
    pub fn own_variable_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        self.visit_in_order(|name, _, _| {
            if !self.hidden_names.contains(name) {
                names.push(name.to_string());
            }
        });
        names
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
                .filter(|name| self.binds(name))
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
        let holds_a_local = |name: &str| {
            !self.hidden_names.contains(name)
                && self
                    .own_value(name)
                    .is_some_and(|value| !names_a_definition_under(name, &value))
        };
        // A scope's own locals come before the ones it can see through the
        // scopes enclosing it, which is the order Ruby reports them in.
        let mut names: Vec<String> = Vec::new();
        self.visit_in_order(|name, _, _| {
            if !self.inherited_names.iter().any(|held| held == name) && holds_a_local(name) {
                names.push(name.to_string());
            }
        });
        names.extend(
            self.inherited_names
                .iter()
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
                if !self.binds(name)
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

    /// Whether this scope is a method's own.
    pub fn is_method_boundary(&self) -> bool {
        self.is_method_boundary
    }

    /// Mark this scope as a method boundary.
    pub fn mark_method_boundary(&mut self) {
        self.version = next_version();
        self.is_method_boundary = true;
    }

    /// The reference this scope holds for `name`, ignoring enclosing scopes.
    pub fn own_var_ref(&self, name: &str) -> Option<Rc<RefCell<Object>>> {
        if let Some(cell) = self.self_ref(name) {
            return Some(Rc::clone(cell));
        }
        self.variables
            .get(name)
            .cloned()
            .or_else(|| self.frame_ref(name))
            .or_else(|| self.captured_ref(name).cloned())
    }

    /// Put `value` in the binding this scope holds for `name` itself,
    /// answering whether it holds one.
    pub fn assign_own(&mut self, name: &str, value: Object) -> bool {
        match self
            .self_ref(name)
            .cloned()
            .or_else(|| self.variables.get(name).cloned())
            .or_else(|| self.frame_ref(name))
        {
            Some(cell) => {
                *cell.borrow_mut() = value;
                true
            }
            None => false,
        }
    }

    /// Whether this scope binds or captured `name` itself.
    pub fn holds_own(&self, name: &str) -> bool {
        self.binds(name) || self.captured_ref(name).is_some()
    }

    /// The value this scope binds `name` to itself, without reading through
    /// what it captured or its parent.
    pub fn own_value(&self, name: &str) -> Option<Object> {
        if let Some(cell) = self.self_ref(name) {
            return Some(cell.borrow().clone());
        }
        self.variables
            .get(name)
            .map(|held| held.borrow().clone())
            .or_else(|| self.frame_value(name))
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
        self.visit_in_order(|name, cell, hoisted| {
            // A name the program never declared is not one a block closes
            // over: a block reading `it` takes the argument it was handed
            // rather than the `it` of the block around it.
            if let Some(cell) = cell
                && !hoisted
                && !self.hidden_names.contains(name)
            {
                take(name, cell);
            }
        });
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
        // A `def` registers its method under its own name, unbound. A Method
        // held under another name, or bound to a receiver, is a local's value.
        View::Method(method) => method.name == name && method.receiver.is_none(),
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
