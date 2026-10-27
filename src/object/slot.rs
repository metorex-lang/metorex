// The name an entry of a Hash is stored under.

use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

#[cfg(not(target_endian = "little"))]
compile_error!("a Slot tells its two forms apart by the first byte of its address");

/// The most bytes a slot holds in place.
const HELD_IN_PLACE: usize = 15;

/// The slot a key takes in a Hash: up to `HELD_IN_PLACE` bytes held in
/// place, with the first byte twice their count plus one, or the address
/// and length of an `Rc<str>`, whose address is even since the text follows
/// the two counts of its allocation.
#[repr(C, align(8))]
pub struct Slot {
    bytes: [u8; 16],
    _shares_an_rc: PhantomData<Rc<str>>,
}

impl Slot {
    fn in_place(text: &str) -> Self {
        let mut bytes = [0u8; 16];
        bytes[0] = ((text.len() as u8) << 1) | 1;
        bytes[1..=text.len()].copy_from_slice(text.as_bytes());
        Slot {
            bytes,
            _shares_an_rc: PhantomData,
        }
    }

    fn shared(text: Rc<str>) -> Self {
        let length = text.len();
        let address = Rc::into_raw(text) as *const u8 as usize;
        debug_assert_eq!(address & 1, 0, "the text of an Rc follows its counts");
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&address.to_le_bytes());
        bytes[8..].copy_from_slice(&length.to_le_bytes());
        Slot {
            bytes,
            _shares_an_rc: PhantomData,
        }
    }

    fn is_in_place(&self) -> bool {
        self.bytes[0] & 1 == 1
    }

    /// The `Rc<str>` a shared slot holds, as its raw pointer.
    fn shared_text(&self) -> *const str {
        let address = usize::from_le_bytes(self.bytes[..8].try_into().expect("eight bytes"));
        let length = usize::from_le_bytes(self.bytes[8..].try_into().expect("eight bytes"));
        std::ptr::slice_from_raw_parts(address as *const u8, length) as *const str
    }

    pub fn as_str(&self) -> &str {
        if self.is_in_place() {
            let length = usize::from(self.bytes[0] >> 1);
            // SAFETY: the bytes were copied from a str of that length.
            unsafe { std::str::from_utf8_unchecked(&self.bytes[1..=length]) }
        } else {
            // SAFETY: the slot holds a count of the Rc until it is dropped.
            unsafe { &*self.shared_text() }
        }
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        if !self.is_in_place() {
            // SAFETY: the pointer came from `Rc::into_raw`, and this slot
            // holds one of its counts.
            drop(unsafe { Rc::from_raw(self.shared_text()) });
        }
    }
}

impl Clone for Slot {
    fn clone(&self) -> Self {
        if !self.is_in_place() {
            // SAFETY: the pointer came from `Rc::into_raw` and is alive.
            unsafe { Rc::increment_strong_count(self.shared_text()) };
        }
        Slot {
            bytes: self.bytes,
            _shares_an_rc: PhantomData,
        }
    }
}

impl Default for Slot {
    fn default() -> Self {
        Slot::in_place("")
    }
}

impl Deref for Slot {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Slot {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Hash for Slot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialEq for Slot {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Slot {}

impl PartialOrd for Slot {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Slot {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl fmt::Debug for Slot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), formatter)
    }
}

impl fmt::Display for Slot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<&str> for Slot {
    fn from(text: &str) -> Self {
        if text.len() <= HELD_IN_PLACE {
            Slot::in_place(text)
        } else {
            Slot::shared(text.into())
        }
    }
}

impl From<String> for Slot {
    fn from(text: String) -> Self {
        if text.len() <= HELD_IN_PLACE {
            Slot::in_place(&text)
        } else {
            Slot::shared(text.into())
        }
    }
}

impl From<&String> for Slot {
    fn from(text: &String) -> Self {
        text.as_str().into()
    }
}

impl From<Rc<str>> for Slot {
    fn from(text: Rc<str>) -> Self {
        if text.len() <= HELD_IN_PLACE {
            Slot::in_place(&text)
        } else {
            Slot::shared(text)
        }
    }
}

impl From<Slot> for String {
    fn from(slot: Slot) -> Self {
        slot.as_str().to_string()
    }
}

impl PartialEq<str> for Slot {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Slot {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for Slot {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<Slot> for str {
    fn eq(&self, other: &Slot) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<Slot> for &str {
    fn eq(&self, other: &Slot) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<Slot> for String {
    fn eq(&self, other: &Slot) -> bool {
        self.as_str() == other.as_str()
    }
}
