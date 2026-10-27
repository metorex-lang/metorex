// The slot a key occupies, and the key object kept beside it.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// The slot an object occupies in a hash. A key that carries its own
    /// `#hash` is placed by that number and then told apart from anything
    /// sharing it by `#eql?`, which is how Ruby decides whether two keys name
    /// the same entry. A hash comparing by identity places every key by its
    /// object id instead, so two equal strings stay two entries.
    pub(crate) fn dict_slot_in(
        &mut self,
        pairs: &crate::object::HashEntries,
        key: &Object,
        by_identity: bool,
        position: Position,
    ) -> Result<crate::object::Slot, MetorexError> {
        if by_identity {
            let id = self.object_identity(key, position)?;
            return Ok(format!("{HASHED_SLOT_PREFIX}i{id}").into());
        }
        if !self.key_hashes_for_itself(key) {
            return Ok((crate::vm::utils::object_to_dict_key(key)
                .unwrap_or_default()
                .to_string())
            .into());
        }
        self.hashed_slot(key, position, |candidate| {
            pairs
                .contains_key(candidate)
                .then(|| reconstruct_key(pairs, candidate))
        })
    }

    /// The slot a key that answers `#hash` takes: the first slot under its
    /// hash number that is free or holds an `#eql?` key. `stored_at` gives
    /// the key held in a slot, or None for a free one.
    pub(crate) fn hashed_slot(
        &mut self,
        key: &Object,
        position: Position,
        stored_at: impl Fn(&str) -> Option<Object>,
    ) -> Result<crate::object::Slot, MetorexError> {
        let hashed = match self
            .send_to_object(key.clone(), "hash", vec![], position)?
            .view()
        {
            View::Int(number) => number,
            other => self.object_identity(&other.to_object(), position)?,
        };
        let mut slot = 0usize;
        loop {
            let candidate = format!("{HASHED_SLOT_PREFIX}h{hashed}#{slot}");
            let Some(stored) = stored_at(&candidate) else {
                return Ok(candidate.into());
            };
            // A key looks itself up without being asked, so a class whose
            // `#eql?` refuses its own object still finds its entry.
            if crate::vm::native_methods::object_methods::same_object(&stored, key) {
                return Ok(candidate.into());
            }
            let same = self
                .send_to_object(key.clone(), "eql?", vec![stored], position)?
                .is_truthy();
            if same {
                return Ok(candidate.into());
            }
            slot += 1;
        }
    }

    /// The slot an object occupies in a live hash, reading the hash's own
    /// identity setting.
    pub(crate) fn dict_slot_for(
        &mut self,
        dict_rc: &Rc<RefCell<crate::object::HashEntries>>,
        key: &Object,
        position: Position,
    ) -> Result<crate::object::Slot, MetorexError> {
        let by_identity = dict_rc.borrow().contains_key(BY_IDENTITY_KEY);
        if by_identity || !self.key_hashes_for_itself(key) {
            return self.dict_slot_in(&Default::default(), key, by_identity, position);
        }
        // `#hash` and `#eql?` run program code that may change the hash, so
        // each slot is read under a borrow of its own.
        self.hashed_slot(key, position, |candidate| {
            let dict = dict_rc.borrow();
            dict.contains_key(candidate)
                .then(|| reconstruct_key(&dict, candidate))
        })
    }

    /// Whether a key answers `#hash` and `#eql?` of its own, which is what
    /// makes the bucket-and-compare placement the right one for it. Anything
    /// else is placed by how it renders, which is cheaper and keeps the
    /// primitives reading back from their slot names.
    fn key_hashes_for_itself(&mut self, key: &Object) -> bool {
        if matches!(key.view(), View::Instance(_)) {
            return true;
        }
        // Ruby places the immediates by value without asking them, so a
        // `TrueClass#hash` written in the program is never reached.
        if matches!(
            key.view(),
            View::Bool(_)
                | View::Int(_)
                | View::BigInt(_)
                | View::Float(_)
                | View::String(_)
                | View::Symbol(_)
                | View::Nil
        ) {
            return false;
        }
        // Every other value answers `#hash` the way its `#eql?` compares, an
        // Array by its elements and a Range by its ends, so it is placed by
        // its hash too.
        self.responds_to(key, "hash")
    }

    /// The object id a hash comparing by identity places a key by. It is
    /// BasicObject's `__id__`, which every object has, as against
    /// `object_id`, which a BasicObject does not.
    fn object_identity(&mut self, key: &Object, position: Position) -> Result<i64, MetorexError> {
        match self
            .send_to_object(key.clone(), "__id__", vec![], position)?
            .view()
        {
            View::Int(number) => Ok(number),
            _ => Ok(0),
        }
    }

    /// Whether two hashes hold the same entries. Each of the receiver's keys
    /// is looked up in the other hash the way any key is, and the values are
    /// compared the strict way for `eql?` and the ordinary way for `==`.
    pub(crate) fn hash_entries_match(
        &mut self,
        dict_rc: &Rc<RefCell<crate::object::HashEntries>>,
        other_rc: &Rc<RefCell<crate::object::HashEntries>>,
        method_name: &str,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let ours = self.hash_pairs(dict_rc);
        let theirs = self.hash_pairs(other_rc);
        if ours.len() != theirs.len() {
            return Ok(false);
        }
        let comparison = if method_name == "eql?" { "eql?" } else { "==" };
        for (key, value) in ours {
            let Some(slot) = self.hash_find_key(other_rc, &key, position)? else {
                return Ok(false);
            };
            let Some(held) = other_rc.borrow().get(slot.as_str()).cloned() else {
                return Ok(false);
            };
            let same = self
                .send_to_object(value, comparison, vec![held], position)?
                .is_truthy();
            if !same {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The key and value of every entry, with the sentinels left out.
    pub(crate) fn hash_pairs_for_digest(
        &self,
        dict_rc: &Rc<RefCell<crate::object::HashEntries>>,
    ) -> Vec<(Object, Object)> {
        self.hash_pairs(dict_rc)
    }

    pub(crate) fn hash_pairs(
        &self,
        dict_rc: &Rc<RefCell<crate::object::HashEntries>>,
    ) -> Vec<(Object, Object)> {
        let dict = dict_rc.borrow();
        dict.iter()
            .filter(|(key, _)| !is_internal_key(key))
            .map(|(key, value)| (reconstruct_key(&dict, key), value.clone()))
            .collect()
    }

    /// The value of every entry, with the sentinels left out.
    pub(crate) fn hash_values(
        &self,
        dict_rc: &Rc<RefCell<crate::object::HashEntries>>,
    ) -> Vec<Object> {
        dict_rc
            .borrow()
            .iter()
            .filter(|(key, _)| !is_internal_key(key))
            .map(|(_, value)| value.clone())
            .collect()
    }

    /// Store one entry, recording the key object beside it. A hash that
    /// already holds a matching key keeps the object it was given the first
    /// time, which is what `keys` reports afterwards.
    pub(crate) fn hash_store(
        &mut self,
        dict_rc: &Rc<RefCell<crate::object::HashEntries>>,
        key: &Object,
        value: Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let rendered = self.dict_slot_for(dict_rc, key, position)?;
        let mut dict = dict_rc.borrow_mut();
        let already_held = dict.contains_key(rendered.as_str());
        if !already_held {
            // A hash comparing by identity keeps the key it was handed, since
            // a copy would be a different object and never found again.
            let stored = if rendered.starts_with("\u{1}i") {
                key.clone()
            } else {
                stored_key_object(key)
            };
            if !crate::vm::utils::is_primitive_key(key)
                || rendered.starts_with(HASHED_SLOT_PREFIX)
                || matches!(stored.view(), View::String(_))
            {
                remember_key_object(&mut dict, &rendered, &stored);
            }
        }
        dict.insert(rendered, value);
        Ok(())
    }

    /// The stored key string matching `wanted`, or None when the hash holds
    /// no entry for it. A key that is not a primitive is matched the way Ruby
    /// matches one: same `hash`, then `eql?`.
    pub(crate) fn hash_find_key(
        &mut self,
        dict_rc: &Rc<RefCell<crate::object::HashEntries>>,
        wanted: &Object,
        position: Position,
    ) -> Result<Option<String>, MetorexError> {
        Ok(self.hash_locate_key(dict_rc, wanted, position)?.1)
    }

    /// The slot a key would take, and the slot it already occupies when the
    /// hash holds it. Both come back from one walk so a key is asked for its
    /// `#hash` once per lookup.
    pub(crate) fn hash_locate_key(
        &mut self,
        dict_rc: &Rc<RefCell<crate::object::HashEntries>>,
        wanted: &Object,
        position: Position,
    ) -> Result<(String, Option<String>), MetorexError> {
        let slot = self.dict_slot_for(dict_rc, wanted, position)?;
        if dict_rc.borrow().contains_key(slot.as_str()) {
            return Ok((slot.clone().to_string(), Some(slot.to_string())));
        }
        // A hash comparing by identity places every key by its object id.
        if dict_rc.borrow().contains_key(BY_IDENTITY_KEY) {
            return Ok((slot.to_string(), None));
        }
        // A hash built outside the program, such as the environment, holds
        // keys placed by how they render rather than by `#hash`, and the
        // environment places one by the text it names. Those are found by
        // walking the keys the hash recorded.
        let rendered = crate::vm::utils::object_to_dict_key(wanted).unwrap_or_default();
        if dict_rc.borrow().contains_key(rendered.as_str()) {
            return Ok((slot.to_string(), Some(rendered.to_string())));
        }
        if crate::vm::utils::is_primitive_key(wanted) {
            return Ok((slot.to_string(), None));
        }
        // Every key that is not a primitive was placed by its `#hash`, so
        // the slots under that hash were the only places it could be.
        if !dict_rc.borrow().contains_key(RENDERED_OBJECTS_KEY) {
            return Ok((slot.to_string(), None));
        }
        let stored: Vec<(crate::object::Slot, Object)> = {
            let dict = dict_rc.borrow();
            dict.keys()
                .filter(|slot| !is_internal_key(slot) && !slot.starts_with(HASHED_SLOT_PREFIX))
                .map(|slot| (slot.clone(), reconstruct_key(&dict, slot)))
                .collect()
        };
        for (held, candidate) in stored {
            if crate::vm::native_methods::object_methods::same_object(&candidate, wanted) {
                return Ok((slot.clone().to_string(), Some(held.to_string())));
            }
            if crate::vm::utils::is_primitive_key(&candidate) {
                continue;
            }
            let same = self
                .send_to_object(wanted.clone(), "eql?", vec![candidate], position)?
                .is_truthy();
            if same {
                return Ok((slot.clone().to_string(), Some(held.to_string())));
            }
        }
        Ok((slot.to_string(), None))
    }
}

/// The object a hash keeps for a key it was handed. Ruby stores a String key
/// as a frozen copy, so a later write through the original text leaves the
/// hash alone; a key already frozen is kept as it stands.
pub(crate) fn stored_key_object(key: &Object) -> Object {
    let View::String(text) = key.view() else {
        return key.clone();
    };
    if text.is_frozen() {
        return key.clone();
    }
    let copy = Object::string(text.as_str().to_string());
    if let View::String(copied) = copy.view() {
        copied.freeze();
    }
    copy
}

/// The prefix on every slot name a key's own `#hash` produced, chosen from
/// the control range so no rendered key can collide with one.
const HASHED_SLOT_PREFIX: char = '\u{1}';

/// A hash key that is not a primitive is recorded in the sentinel sub-map, so
/// the original object comes back when the hash is walked.
pub(crate) fn remember_key_object(
    pairs: &mut crate::object::HashEntries,
    rendered: &str,
    key: &Object,
) {
    if !rendered.starts_with(HASHED_SLOT_PREFIX) && !crate::vm::utils::is_primitive_key(key) {
        pairs.insert(RENDERED_OBJECTS_KEY.into(), Object::Bool(true));
    }
    // A copy of a hash shares the record with the hash it came from, so a
    // shared record is copied before it changes, and one held by this hash
    // alone is written in place.
    if let Some(record) = pairs.get(KEY_OBJECTS_KEY)
        && record.references() == 1
        && let View::Dict(existing) = record.view()
    {
        existing
            .borrow_mut()
            .insert(rendered.to_string().into(), key.clone());
        return;
    }
    let mut objects = match pairs.get(KEY_OBJECTS_KEY).view() {
        Some(View::Dict(existing)) => existing.borrow().clone(),
        _ => Default::default(),
    };
    objects.insert(rendered.to_string().into(), key.clone());
    pairs.insert(
        KEY_OBJECTS_KEY.into(),
        Object::Dict(Rc::new(RefCell::new(objects))),
    );
}

/// Drop the key object recorded for `slot`. A record shared with a copy of
/// the hash is copied first, so the copy keeps its keys.
pub(crate) fn forget_key_object(pairs: &mut crate::object::HashEntries, slot: &str) {
    let Some(record) = pairs.get(KEY_OBJECTS_KEY) else {
        return;
    };
    let View::Dict(existing) = record.view() else {
        return;
    };
    if record.references() == 1 {
        existing.borrow_mut().shift_remove(slot);
        return;
    }
    let mut objects = existing.borrow().clone();
    objects.shift_remove(slot);
    pairs.insert(
        KEY_OBJECTS_KEY.into(),
        Object::Dict(Rc::new(RefCell::new(objects))),
    );
}

/// Record one entry in a hash being rebuilt, keeping the key object beside it
/// when the key does not read back from its rendering.
pub(crate) fn keep_pair(built: &mut crate::object::HashEntries, key: Object, value: Object) {
    let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
    if !crate::vm::utils::is_primitive_key(&key) {
        remember_key_object(built, &rendered, &key);
    }
    built.insert(rendered, value);
}
