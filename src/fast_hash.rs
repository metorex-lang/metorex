//! A hasher for the interpreter's own tables, keyed by names written in the
//! program's source: methods, constants, and variables. It mixes a word at a
//! time, which is much less work than the standard library's SipHash. A Ruby
//! Hash keeps SipHash, since its keys come from the program's data.

use std::cell::RefCell;
use std::hash::{BuildHasherDefault, Hasher};
use std::rc::Rc;

/// The multiplier FxHash mixes each word with.
const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// FxHash, as rustc uses it for its own tables.
#[derive(Default, Clone, Copy)]
pub struct NameHasher {
    hash: u64,
}

impl NameHasher {
    fn add_word(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for NameHasher {
    fn write(&mut self, bytes: &[u8]) {
        let (words, rest) = bytes.as_chunks::<8>();
        for word in words {
            self.add_word(u64::from_le_bytes(*word));
        }
        if !rest.is_empty() {
            let mut word = [0u8; 8];
            word[..rest.len()].copy_from_slice(rest);
            self.add_word(u64::from_le_bytes(word));
        }
    }

    fn write_u8(&mut self, value: u8) {
        self.add_word(u64::from(value));
    }

    fn write_usize(&mut self, value: usize) {
        self.add_word(value as u64);
    }

    fn write_u64(&mut self, value: u64) {
        self.add_word(value);
    }

    fn finish(&self) -> u64 {
        self.hash
    }
}

/// What builds a `NameHasher` for a map.
pub type NameHashing = BuildHasherDefault<NameHasher>;

/// A HashMap keyed by names from the program's source.
pub type NameMap<K, V> = std::collections::HashMap<K, V, NameHashing>;

/// A HashSet of names from the program's source.
pub type NameSet<K> = std::collections::HashSet<K, NameHashing>;

/// An IndexMap keyed by names from the program's source.
pub type NameIndexMap<K, V> = indexmap::IndexMap<K, V, NameHashing>;

thread_local! {
    /// The names method caches and closed-over variables are keyed by, each
    /// held once.
    static SHARED_NAMES: RefCell<NameSet<Rc<str>>> = RefCell::new(NameSet::default());
}

/// `name` as the method caches and closed-over variables hold it.
pub fn shared_name(name: &str) -> Rc<str> {
    SHARED_NAMES.with(|names| {
        let mut names = names.borrow_mut();
        if let Some(held) = names.get(name) {
            return Rc::clone(held);
        }
        let held: Rc<str> = name.into();
        names.insert(Rc::clone(&held));
        held
    })
}

thread_local! {
    /// The random keys every Ruby Hash on this thread hashes its keys with.
    static PROGRAM_KEYS: std::collections::hash_map::RandomState =
        std::collections::hash_map::RandomState::new();
}

/// What builds the hasher of a Ruby Hash: SipHash under keys chosen at
/// random once a thread, kept outside the Hash so it adds nothing to it.
#[derive(Clone, Copy, Default, Debug)]
pub struct ProgramHashing;

impl std::hash::BuildHasher for ProgramHashing {
    type Hasher = std::collections::hash_map::DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        PROGRAM_KEYS.with(std::hash::BuildHasher::build_hasher)
    }
}
