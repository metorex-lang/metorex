// Slot - the name a Hash entry is stored under

use metorex::object::Slot;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hashed<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn a_slot_takes_two_words() {
    assert_eq!(std::mem::size_of::<Slot>(), 16);
}

#[test]
fn a_short_slot_and_a_long_one_read_back_the_text_they_were_made_from() {
    for text in [
        "",
        ":name",
        "fifteen chars!!",
        "sixteen chars!!!",
        ":a_much_longer_symbol_key",
    ] {
        assert_eq!(Slot::from(text).as_str(), text);
    }
}

#[test]
fn a_slot_hashes_and_compares_as_its_text_does() {
    for text in [":name", ":a_much_longer_symbol_key"] {
        let slot = Slot::from(text.to_string());
        assert_eq!(hashed(&slot), hashed(text));
        assert!(slot == text);
    }
}

#[test]
fn a_copy_of_a_long_slot_outlives_the_slot_it_was_copied_from() {
    let first = Slot::from(":a_much_longer_symbol_key");
    let copy = first.clone();
    drop(first);
    assert_eq!(copy.as_str(), ":a_much_longer_symbol_key");
}

#[test]
fn a_hash_keyed_by_slots_finds_an_entry_by_its_text() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert(Slot::from(":name"), 1);
    entries.insert(Slot::from(":a_much_longer_symbol_key"), 2);
    assert_eq!(entries.get(":name"), Some(&1));
    assert_eq!(entries.get(":a_much_longer_symbol_key"), Some(&2));
}
