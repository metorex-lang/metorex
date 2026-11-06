// The objects OpenSSL names: each one's short name, long name, and OID, as
// `openssl list -objects` prints OpenSSL's built-in table.

use super::*;
use crate::object::{OptionView, View};
use std::collections::HashMap;
use std::sync::OnceLock;

/// One object per line: the short name, the long name, and the OID, with a
/// tab between them. An object with no OID has an empty third field.
const OBJECT_TABLE: &str = include_str!("../../stdlib/openssl_objects.tsv");

/// One object of the table: its short name, long name, and OID.
type ObjectRow = (&'static str, &'static str, &'static str);

/// The table's rows, with the first row each short name, long name, and OID
/// appears in, built the first time a name is looked up.
struct ObjectIndex {
    rows: Vec<ObjectRow>,
    by_short: HashMap<&'static str, usize>,
    by_long: HashMap<&'static str, usize>,
    by_oid: HashMap<&'static str, usize>,
}

fn object_index() -> &'static ObjectIndex {
    static INDEX: OnceLock<ObjectIndex> = OnceLock::new();
    INDEX.get_or_init(|| {
        let rows: Vec<ObjectRow> = OBJECT_TABLE
            .lines()
            .filter_map(|line| {
                let mut fields = line.split('\t');
                Some((
                    fields.next()?,
                    fields.next()?,
                    fields.next().unwrap_or_default(),
                ))
            })
            .collect();
        let mut index = ObjectIndex {
            by_short: HashMap::new(),
            by_long: HashMap::new(),
            by_oid: HashMap::new(),
            rows: Vec::new(),
        };
        for (position, (short, long, oid)) in rows.iter().enumerate() {
            index.by_short.entry(short).or_insert(position);
            index.by_long.entry(long).or_insert(position);
            if !oid.is_empty() {
                index.by_oid.entry(oid).or_insert(position);
            }
        }
        index.rows = rows;
        index
    })
}

/// The short name, long name, and OID of the object a name stands for,
/// looked up the way OBJ_txt2nid looks it up: by short name, then by long
/// name, then by OID.
fn object_named(text: &str) -> Option<ObjectRow> {
    let index = object_index();
    index
        .by_short
        .get(text)
        .or_else(|| index.by_long.get(text))
        .or_else(|| index.by_oid.get(text))
        .map(|position| index.rows[*position])
}

/// Whether text is a dotted OID OpenSSL would take: at least two arcs of
/// digits, the first of them 0, 1, or 2.
fn dotted_oid(text: &str) -> bool {
    let arcs: Vec<&str> = text.split('.').collect();
    arcs.len() >= 2
        && arcs
            .iter()
            .all(|arc| !arc.is_empty() && arc.bytes().all(|byte| byte.is_ascii_digit()))
        && matches!(arcs[0], "0" | "1" | "2")
}

/// The DER content of an OID: the first two arcs folded into one byte's
/// worth, then each arc in base 128 with the high bit marking continuation.
fn encoded_oid(oid: &str) -> Vec<u8> {
    let arcs: Vec<u128> = oid.split('.').filter_map(|arc| arc.parse().ok()).collect();
    let mut bytes = Vec::new();
    let mut push = |mut value: u128| {
        let mut held = vec![(value & 0x7f) as u8];
        value >>= 7;
        while value > 0 {
            held.push(((value & 0x7f) as u8) | 0x80);
            value >>= 7;
        }
        held.reverse();
        bytes.extend(held);
    };
    push(arcs[0] * 40 + arcs.get(1).copied().unwrap_or(0));
    for arc in arcs.iter().skip(2) {
        push(*arc);
    }
    bytes
}

impl VirtualMachine {
    /// `OpenSSL.__object__(:find, text)` answers the short name, long name,
    /// and OID an object is known by, or nil for none. A dotted OID the
    /// table does not hold names itself in all three places.
    /// `OpenSSL.__object__(:encode, oid)` answers the DER content of an OID.
    pub(crate) fn openssl_object_command(&mut self, arguments: &[Object]) -> Object {
        let text = match arguments.get(1).view() {
            Some(View::String(held)) => held.as_str().to_string(),
            _ => return Object::Nil,
        };
        match arguments.first().view() {
            Some(View::Symbol(command)) if &*command.as_str() == "encode" => {
                if !dotted_oid(&text) {
                    return Object::Nil;
                }
                crate::vm::native_methods::pack_format::bytes_to_string(&encoded_oid(&text))
            }
            _ => match object_named(&text) {
                Some((short, long, oid)) => Object::array(vec![
                    Object::string(short.to_string()),
                    Object::string(long.to_string()),
                    if oid.is_empty() {
                        Object::Nil
                    } else {
                        Object::string(oid.to_string())
                    },
                ]),
                None if dotted_oid(&text) => Object::array(vec![
                    Object::string(text.clone()),
                    Object::string(text.clone()),
                    Object::string(text),
                ]),
                None => Object::Nil,
            },
        }
    }
}
