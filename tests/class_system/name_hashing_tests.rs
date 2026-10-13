// The hasher the interpreter's method, constant, and variable tables use.

use metorex::fast_hash::{NameHashing, NameMap};
use std::hash::BuildHasher;

fn hash_of(name: &str) -> u64 {
    NameHashing::default().hash_one(name)
}

#[test]
fn the_same_name_hashes_the_same_every_time() {
    assert_eq!(hash_of("define_method"), hash_of("define_method"));
}

#[test]
fn names_differing_in_one_letter_hash_differently() {
    let names = ["total", "totals", "Total", "tota", "subtotal", "total="];
    let mut hashes: Vec<u64> = names.iter().map(|name| hash_of(name)).collect();
    hashes.sort_unstable();
    hashes.dedup();
    assert_eq!(hashes.len(), names.len());
}

#[test]
fn a_name_map_finds_every_name_stored_in_it() {
    let mut table = NameMap::default();
    for index in 0..1000 {
        table.insert(format!("method_{index}"), index);
    }
    assert!((0..1000).all(|index| table.get(&format!("method_{index}")) == Some(&index)));
}
