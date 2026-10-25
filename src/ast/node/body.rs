// The statements a definition holds, which a copy read from the binary
// keeps encoded until something reads them.

use super::Statement;
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

/// The statements of a method, block, class, or module body, which every
/// copy of the definition and the unit it compiles to share.
#[derive(Clone)]
pub struct Body(Rc<Statements>);

struct Statements {
    decoded: OnceCell<Box<[Statement]>>,
    /// The bodies of the copy this one was read from, and its own bytes
    /// among them.
    encoded: Cell<Option<(&'static [u8], &'static [u8])>>,
}

thread_local! {
    /// The bodies written beside the statements being encoded on this
    /// thread.
    static ENCODING: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
    /// The bodies of the copy being decoded on this thread.
    static DECODING: Cell<&'static [u8]> = const { Cell::new(&[]) };
}

/// The bytes `value` encodes to, with every body in it written after the
/// rest, so a reader that decodes only some of the bodies reads only their
/// pages. The first four bytes are the length of the rest.
pub fn encode_held<T: serde::Serialize>(value: &T) -> Vec<u8> {
    ENCODING.with(|bodies| *bodies.borrow_mut() = Some(Vec::new()));
    let outer = rmp_serde::to_vec(value).expect("statements serialize");
    let bodies = ENCODING
        .with(|bodies| bodies.borrow_mut().take())
        .expect("the bodies were being written");
    let mut held = u32::try_from(outer.len())
        .expect("the statements fit in four gigabytes")
        .to_le_bytes()
        .to_vec();
    held.extend(outer);
    held.extend(bodies);
    held
}

/// Decode `held`, which `encode_held` wrote and the binary holds, with each
/// body in it left encoded until something reads it.
pub fn decode_held<T: serde::de::DeserializeOwned>(held: &'static [u8]) -> T {
    let (length, rest) = held.split_at(4);
    let length = u32::from_le_bytes(length.try_into().expect("four bytes")) as usize;
    let (outer, bodies) = rest.split_at(length);
    decode_among(bodies, outer)
}

fn decode_among<T: serde::de::DeserializeOwned>(
    bodies: &'static [u8],
    encoded: &'static [u8],
) -> T {
    let outer = DECODING.replace(bodies);
    let decoded = rmp_serde::from_slice(encoded).expect("the build script wrote this copy");
    DECODING.set(outer);
    decoded
}

impl Body {
    fn decoded(statements: Box<[Statement]>) -> Self {
        Body(Rc::new(Statements {
            decoded: OnceCell::from(statements),
            encoded: Cell::new(None),
        }))
    }
}

impl std::ops::Deref for Body {
    type Target = [Statement];

    fn deref(&self) -> &[Statement] {
        self.0.decoded.get_or_init(|| {
            let (bodies, encoded) = self.0.encoded.take().expect("a body holds its statements");
            let _counted =
                crate::memory_category::attribute(crate::memory_category::Category::SyntaxTree);
            decode_among(bodies, encoded)
        })
    }
}

impl From<Vec<Statement>> for Body {
    fn from(statements: Vec<Statement>) -> Self {
        Body::decoded(statements.into_boxed_slice())
    }
}

impl<const N: usize> From<[Statement; N]> for Body {
    fn from(statements: [Statement; N]) -> Self {
        Body::decoded(Box::new(statements))
    }
}

impl From<&[Statement]> for Body {
    fn from(statements: &[Statement]) -> Self {
        Body::decoded(statements.into())
    }
}

impl PartialEq for Body {
    fn eq(&self, other: &Body) -> bool {
        **self == **other
    }
}

impl std::fmt::Debug for Body {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(formatter)
    }
}

/// A body is written as where its bytes start among the bodies
/// `encode_held` writes and how many there are.
impl serde::Serialize for Body {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let encoded = rmp_serde::to_vec(&**self).map_err(serde::ser::Error::custom)?;
        let start = ENCODING.with(|bodies| {
            let mut bodies = bodies.borrow_mut();
            let bodies = bodies
                .as_mut()
                .ok_or_else(|| serde::ser::Error::custom("a body encoded outside encode_held"))?;
            let start = bodies.len();
            bodies.extend(encoded.iter());
            Ok(start)
        })?;
        let place = |at: usize| u32::try_from(at).map_err(serde::ser::Error::custom);
        (place(start)?, place(encoded.len())?).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Body {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (start, length) = <(u32, u32)>::deserialize(deserializer)?;
        let bodies = DECODING.get();
        let encoded = bodies
            .get(start as usize..start as usize + length as usize)
            .ok_or_else(|| serde::de::Error::custom("a body outside the copy being decoded"))?;
        Ok(Body(Rc::new(Statements {
            decoded: OnceCell::new(),
            encoded: Cell::new(Some((bodies, encoded))),
        })))
    }
}
