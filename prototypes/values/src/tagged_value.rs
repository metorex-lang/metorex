//! An 8-byte value laid out the way CRuby lays out a VALUE: an Integer that
//! fits in 63 bits with its low bit set, a Float whose exponent is in the
//! common range rotated into a "flonum" tagged 0b10, nil, true, and false as
//! fixed constants, and anything else a pointer to a reference-counted cell
//! on the heap. Cloning or dropping a value touches memory only when it is a
//! pointer.

use crate::machine::Representation;
use std::cell::{Cell, RefCell};

const FALSE: u64 = 0x00;
const NIL: u64 = 0x08;
const TRUE: u64 = 0x14;
const FIXNUM_MAX: i64 = i64::MAX >> 1;
const FIXNUM_MIN: i64 = i64::MIN >> 1;
/// The flonum CRuby uses for 0.0, whose bits would otherwise read as false.
const FLONUM_ZERO: u64 = 0x8000_0000_0000_0002;

pub struct Tagged(u64);

enum Body {
    Slots(RefCell<Vec<Tagged>>),
    Float(f64),
    Integer(i128),
}

/// What a pointer value points at. The alignment keeps a pointer's low three
/// bits clear, which is what tells it apart from every immediate value.
#[repr(C, align(8))]
struct HeapCell {
    references: Cell<usize>,
    body: Body,
}

impl Tagged {
    /// The bits a C extension is handed as the VALUE.
    pub fn bits(&self) -> u64 {
        self.0
    }

    /// The value a VALUE C handed back stands for, holding a reference of
    /// its own when it is a pointer.
    pub fn from_bits_taking_reference(bits: u64) -> Tagged {
        let borrowed = std::mem::ManuallyDrop::new(Tagged(bits));
        (*borrowed).clone()
    }

    fn is_fixnum(&self) -> bool {
        self.0 & 1 == 1
    }

    fn is_flonum(&self) -> bool {
        self.0 & 3 == 2
    }

    fn is_pointer(&self) -> bool {
        self.0 & 7 == 0 && self.0 > TRUE
    }

    fn cell(&self) -> &HeapCell {
        debug_assert!(self.is_pointer());
        // SAFETY: a pointer value was made from a leaked Box<HeapCell> and
        // holds one of its references, so the cell is alive.
        unsafe { &*(self.0 as *const HeapCell) }
    }

    fn on_heap(body: Body) -> Tagged {
        let cell = Box::new(HeapCell {
            references: Cell::new(1),
            body,
        });
        Tagged(Box::into_raw(cell) as u64)
    }

    fn fixnum(number: i64) -> Tagged {
        Tagged(((number as u64) << 1) | 1)
    }

    fn fixnum_value(&self) -> i64 {
        (self.0 as i64) >> 1
    }

    fn wide_integer(&self) -> Option<i128> {
        if self.is_fixnum() {
            return Some(self.fixnum_value() as i128);
        }
        if self.is_pointer()
            && let Body::Integer(number) = self.cell().body
        {
            return Some(number);
        }
        None
    }

    fn float_value(&self) -> Option<f64> {
        if self.is_flonum() {
            if self.0 == FLONUM_ZERO {
                return Some(0.0);
            }
            let sign = self.0 >> 63;
            let restored = ((2 - sign) | (self.0 & !3)).rotate_right(3);
            return Some(f64::from_bits(restored));
        }
        if self.is_pointer()
            && let Body::Float(number) = self.cell().body
        {
            return Some(number);
        }
        None
    }

    fn slots(&self) -> &RefCell<Vec<Tagged>> {
        match &self.cell().body {
            Body::Slots(held) => held,
            _ => panic!("the value holds no slots"),
        }
    }
}

impl Clone for Tagged {
    fn clone(&self) -> Tagged {
        if self.is_pointer() {
            let references = &self.cell().references;
            references.set(references.get() + 1);
        }
        Tagged(self.0)
    }
}

impl Drop for Tagged {
    fn drop(&mut self) {
        if !self.is_pointer() {
            return;
        }
        let references = &self.cell().references;
        let left = references.get() - 1;
        references.set(left);
        if left == 0 {
            // SAFETY: this was the last reference to a cell made by
            // `on_heap`, so nothing reads it after it is freed.
            drop(unsafe { Box::from_raw(self.0 as *mut HeapCell) });
        }
    }
}

fn integer_result(wide: i128) -> Tagged {
    match i64::try_from(wide) {
        Ok(number) if (FIXNUM_MIN..=FIXNUM_MAX).contains(&number) => Tagged::fixnum(number),
        _ => Tagged::on_heap(Body::Integer(wide)),
    }
}

fn arithmetic(
    left: Tagged,
    right: Tagged,
    checked: fn(i64, i64) -> Option<i64>,
    wide: fn(i128, i128) -> i128,
    float: fn(f64, f64) -> f64,
) -> Tagged {
    if left.is_fixnum() && right.is_fixnum() {
        // Two fixnums' result fits in an i64 unless the operation overflows,
        // and a result outside the fixnum range goes to the heap.
        return match checked(left.fixnum_value(), right.fixnum_value()) {
            Some(number) if (FIXNUM_MIN..=FIXNUM_MAX).contains(&number) => Tagged::fixnum(number),
            _ => Tagged::on_heap(Body::Integer(wide(
                left.fixnum_value() as i128,
                right.fixnum_value() as i128,
            ))),
        };
    }
    if let (Some(left), Some(right)) = (left.float_value(), right.float_value()) {
        return TaggedValue::float(float(left, right));
    }
    match (left.wide_integer(), right.wide_integer()) {
        (Some(left), Some(right)) => integer_result(wide(left, right)),
        _ => {
            let as_float = |value: &Tagged| {
                value
                    .float_value()
                    .or_else(|| value.wide_integer().map(|number| number as f64))
            };
            match (as_float(&left), as_float(&right)) {
                (Some(left), Some(right)) => TaggedValue::float(float(left, right)),
                _ => panic!("no arithmetic for these values"),
            }
        }
    }
}

thread_local! {
    /// How many Floats outside the flonum range went to the heap.
    pub static HEAP_FLOATS: Cell<usize> = const { Cell::new(0) };
}

pub struct TaggedValue;

impl Representation for TaggedValue {
    type Value = Tagged;

    fn nil() -> Tagged {
        Tagged(NIL)
    }

    fn integer(number: i64) -> Tagged {
        integer_result(number as i128)
    }

    fn float(number: f64) -> Tagged {
        let bits = number.to_bits();
        let top = (bits >> 60) & 7;
        if bits != 0x3000_0000_0000_0000 && (top.wrapping_sub(3) & !1) == 0 {
            return Tagged((bits.rotate_left(3) & !1) | 2);
        }
        if bits == 0 {
            return Tagged(FLONUM_ZERO);
        }
        HEAP_FLOATS.set(HEAP_FLOATS.get() + 1);
        Tagged::on_heap(Body::Float(number))
    }

    fn truthy(value: &Tagged) -> bool {
        value.0 != NIL && value.0 != FALSE
    }

    fn add(left: Tagged, right: Tagged) -> Tagged {
        arithmetic(left, right, i64::checked_add, |a, b| a + b, |a, b| a + b)
    }

    fn subtract(left: Tagged, right: Tagged) -> Tagged {
        arithmetic(left, right, i64::checked_sub, |a, b| a - b, |a, b| a - b)
    }

    fn multiply(left: Tagged, right: Tagged) -> Tagged {
        arithmetic(left, right, i64::checked_mul, |a, b| a * b, |a, b| a * b)
    }

    fn less_than(left: Tagged, right: Tagged) -> Tagged {
        let answer = if left.is_fixnum() && right.is_fixnum() {
            left.fixnum_value() < right.fixnum_value()
        } else {
            match (left.float_value(), right.float_value()) {
                (Some(left), Some(right)) => left < right,
                _ => panic!("no comparison for these values"),
            }
        };
        Tagged(if answer { TRUE } else { FALSE })
    }

    fn new_object(count: usize) -> Tagged {
        let slots = (0..count).map(|_| Tagged(NIL)).collect();
        Tagged::on_heap(Body::Slots(RefCell::new(slots)))
    }

    fn get_slot(object: &Tagged, slot: usize) -> Tagged {
        object.slots().borrow()[slot].clone()
    }

    fn set_slot(object: &Tagged, slot: usize, value: Tagged) {
        object.slots().borrow_mut()[slot] = value;
    }

    fn new_array() -> Tagged {
        Tagged::on_heap(Body::Slots(RefCell::new(Vec::new())))
    }

    fn array_push(array: &Tagged, value: Tagged) {
        array.slots().borrow_mut().push(value);
    }

    fn array_get(array: &Tagged, index: &Tagged) -> Tagged {
        let index = index.fixnum_value() as usize;
        array
            .slots()
            .borrow()
            .get(index)
            .cloned()
            .unwrap_or(Tagged(NIL))
    }

    fn describe(value: &Tagged) -> String {
        if let Some(number) = value.wide_integer() {
            return number.to_string();
        }
        if let Some(number) = value.float_value() {
            return format!("{number:?}");
        }
        match value.0 {
            NIL => "nil".to_string(),
            TRUE => "true".to_string(),
            FALSE => "false".to_string(),
            _ => "an object".to_string(),
        }
    }
}
