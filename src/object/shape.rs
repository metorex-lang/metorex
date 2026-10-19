//! Object shapes: the names of an object's instance variables, in the order
//! they were first assigned, shared by every object that assigned the same
//! names in the same order. An object keeps its values in slots numbered by
//! its shape, so a read site that has seen the shape before reads a slot.

use crate::fast_hash::NameMap;
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use super::Object;

/// How many names a shape searches in order before it keeps a map of them.
const NAMES_SEARCHED_IN_ORDER: usize = 8;

thread_local! {
    static ROOT: Rc<Shape> = Rc::new(Shape::root());
    static SHAPES_MADE: Cell<u64> = const { Cell::new(1) };
}

/// The number of shapes made so far, the shape with no names counted.
pub fn shapes_made() -> u64 {
    SHAPES_MADE.with(Cell::get)
}

/// The names of an object's instance variables. A shape is never freed: the
/// shape it was made from holds it, so its address names it for good.
#[derive(Debug)]
pub struct Shape {
    names: Vec<String>,
    /// The slot of each name, made once the shape holds more names than it
    /// searches in order.
    slots_by_name: OnceCell<NameMap<String, usize>>,
    /// The shape each further name leads to.
    next: RefCell<NameMap<String, Rc<Shape>>>,
}

impl Shape {
    fn root() -> Shape {
        Shape {
            names: Vec::new(),
            slots_by_name: OnceCell::new(),
            next: RefCell::new(NameMap::default()),
        }
    }

    /// The slot `name` is held in.
    pub fn slot_of(&self, name: &str) -> Option<usize> {
        if self.names.len() <= NAMES_SEARCHED_IN_ORDER {
            return self.names.iter().position(|held| held == name);
        }
        self.slots_by_name
            .get_or_init(|| {
                self.names
                    .iter()
                    .enumerate()
                    .map(|(slot, name)| (name.clone(), slot))
                    .collect()
            })
            .get(name)
            .copied()
    }

    /// The shape of an object with this one's names followed by `name`.
    fn with(self: &Rc<Shape>, name: &str) -> Rc<Shape> {
        if let Some(next) = self.next.borrow().get(name) {
            return Rc::clone(next);
        }
        let mut names = self.names.clone();
        names.push(name.to_string());
        let made = Rc::new(Shape {
            names,
            slots_by_name: OnceCell::new(),
            next: RefCell::new(NameMap::default()),
        });
        SHAPES_MADE.with(|count| count.set(count.get() + 1));
        self.next
            .borrow_mut()
            .insert(name.to_string(), Rc::clone(&made));
        made
    }
}

/// An object's instance variables: its shape and the value in each slot.
#[derive(Clone)]
pub struct InstanceVariables {
    shape: Rc<Shape>,
    slots: Vec<Object>,
}

impl Default for InstanceVariables {
    fn default() -> Self {
        InstanceVariables {
            shape: ROOT.with(Rc::clone),
            slots: Vec::new(),
        }
    }
}

impl InstanceVariables {
    pub fn shape(&self) -> &Rc<Shape> {
        &self.shape
    }

    pub fn get(&self, name: &str) -> Option<&Object> {
        self.shape.slot_of(name).map(|slot| &self.slots[slot])
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.shape.slot_of(name).is_some()
    }

    /// Set `name` to `value`, answering the value it replaced.
    pub fn insert(&mut self, name: String, value: Object) -> Option<Object> {
        match self.shape.slot_of(&name) {
            Some(slot) => Some(std::mem::replace(&mut self.slots[slot], value)),
            None => {
                self.shape = self.shape.with(&name);
                self.slots.push(value);
                None
            }
        }
    }

    /// Remove `name`, keeping the order of the names after it.
    pub fn shift_remove(&mut self, name: &str) -> Option<Object> {
        let slot = self.shape.slot_of(name)?;
        let removed = self.slots.remove(slot);
        let mut shape = ROOT.with(Rc::clone);
        for (index, kept) in self.shape.names.iter().enumerate() {
            if index != slot {
                shape = shape.with(kept);
            }
        }
        self.shape = shape;
        Some(removed)
    }

    /// The value in `slot`, which the shape names.
    pub fn slot(&self, slot: usize) -> &Object {
        &self.slots[slot]
    }

    /// Add `value` in a new last slot, where `shape`, the shape this one
    /// leads to by the name the slot holds, names it.
    pub fn push_into_shape(&mut self, shape: Rc<Shape>, value: Object) {
        debug_assert!(
            shape.names.len() == self.slots.len() + 1
                && shape.names[..self.slots.len()] == self.shape.names[..],
            "a shape one name past the object's own"
        );
        self.shape = shape;
        self.slots.push(value);
    }

    /// Replace the value in `slot`, which the shape names.
    pub fn set_slot(&mut self, slot: usize, value: Object) {
        self.slots[slot] = value;
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn keys(&self) -> std::slice::Iter<'_, String> {
        self.shape.names.iter()
    }

    pub fn values(&self) -> std::slice::Iter<'_, Object> {
        self.slots.iter()
    }

    pub fn iter(
        &self,
    ) -> std::iter::Zip<std::slice::Iter<'_, String>, std::slice::Iter<'_, Object>> {
        self.shape.names.iter().zip(self.slots.iter())
    }

    pub fn into_values(self) -> std::vec::IntoIter<Object> {
        self.slots.into_iter()
    }
}

impl<'a> IntoIterator for &'a InstanceVariables {
    type Item = (&'a String, &'a Object);
    type IntoIter = std::iter::Zip<std::slice::Iter<'a, String>, std::slice::Iter<'a, Object>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Two objects' variables are equal when they hold the same names with
/// equal values, whatever order the names were assigned in.
impl PartialEq for InstanceVariables {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .all(|(name, value)| other.get(name) == Some(value))
    }
}

impl std::fmt::Debug for InstanceVariables {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_map().entries(self.iter()).finish()
    }
}

/// The shape an instance variable site last saw and the slot its name has
/// there.
#[derive(Debug)]
pub struct SlotCache {
    seen: Cell<(*const Shape, usize)>,
}

impl Default for SlotCache {
    fn default() -> Self {
        SlotCache {
            seen: Cell::new((std::ptr::null(), 0)),
        }
    }
}

impl SlotCache {
    /// The shape this site saw last and the slot its name has there.
    pub fn seen(&self) -> Option<(*const Shape, usize)> {
        let (shape, slot) = self.seen.get();
        (!shape.is_null()).then_some((shape, slot))
    }

    /// The slot of `name` in `variables`, answered without a search when
    /// their shape is the one this site saw last.
    pub fn slot_in(&self, variables: &InstanceVariables, name: &str) -> Option<usize> {
        let (shape, slot) = self.seen.get();
        if std::ptr::eq(shape, Rc::as_ptr(&variables.shape)) {
            return Some(slot);
        }
        let slot = variables.shape.slot_of(name)?;
        self.seen.set((Rc::as_ptr(&variables.shape), slot));
        Some(slot)
    }
}
