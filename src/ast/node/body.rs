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
    /// Whether the first statement was written by the program rather than
    /// in the prelude.
    opens_in_the_program: bool,
    count: u32,
    /// The bodies of the copy this one was read from, and its own bytes
    /// among them.
    encoded: Cell<Option<(&'static HeldAreas, &'static [u8])>>,
    /// What every method defined with the statements shares, which is
    /// where they keep what the statements compile to.
    shared_by_methods: OnceCell<Rc<dyn std::any::Any>>,
}

/// The bodies being written for `encode_held`, in the two areas of
/// `HeldAreas`.
#[derive(Default)]
struct WrittenAreas {
    at_load: Vec<u8>,
    when_called: Vec<u8>,
}

/// The bodies of a copy `encode_held` wrote, in two areas: the bodies that
/// run as the copy loads, and the bodies of the methods it defines, which
/// run only when a method is called. A load reads only pages of the first.
struct HeldAreas {
    at_load: &'static [u8],
    when_called: &'static [u8],
}

thread_local! {
    /// The bodies written beside the statements being encoded on this
    /// thread.
    static ENCODING: RefCell<Option<WrittenAreas>> = const { RefCell::new(None) };
    /// Whether the body being encoded on this thread is inside a method's.
    static IN_A_METHOD: Cell<bool> = const { Cell::new(false) };
    /// The bodies of the copy being decoded on this thread.
    static DECODING: Cell<Option<&'static HeldAreas>> = const { Cell::new(None) };
}

/// The bytes `value` encodes to, with every body in it written after the
/// rest, so a reader that decodes only some of the bodies reads only their
/// pages. The first eight bytes are the lengths of the rest and of the
/// bodies that run as the copy loads.
pub fn encode_held<T: serde::Serialize>(value: &T) -> Vec<u8> {
    ENCODING.with(|bodies| *bodies.borrow_mut() = Some(WrittenAreas::default()));
    let outer = rmp_serde::to_vec(value).expect("statements serialize");
    let areas = ENCODING
        .with(|bodies| bodies.borrow_mut().take())
        .expect("the bodies were being written");
    let length = |bytes: &[u8]| {
        u32::try_from(bytes.len())
            .expect("the statements fit in four gigabytes")
            .to_le_bytes()
    };
    let mut held = length(&outer).to_vec();
    held.extend(length(&areas.at_load));
    held.extend(outer);
    held.extend(areas.at_load);
    held.extend(areas.when_called);
    held
}

/// Decode `held`, which `encode_held` wrote and the binary holds, with each
/// body in it left encoded until something reads it.
pub fn decode_held<T: serde::de::DeserializeOwned>(held: &'static [u8]) -> T {
    let (areas, outer) = held_areas(held);
    decode_among(areas, outer)
}

/// The areas of the bodies in `held`, which `encode_held` wrote, and the
/// bytes of what holds them.
fn held_areas(held: &'static [u8]) -> (&'static HeldAreas, &'static [u8]) {
    let length =
        |at: usize| u32::from_le_bytes(held[at..at + 4].try_into().expect("four bytes")) as usize;
    let (outer_length, at_load_length) = (length(0), length(4));
    let (outer, bodies) = held[8..].split_at(outer_length);
    let (at_load, when_called) = bodies.split_at(at_load_length);
    // A copy lives as long as the binary, so the record of its areas does.
    let areas: &'static HeldAreas = Box::leak(Box::new(HeldAreas {
        at_load,
        when_called,
    }));
    (areas, outer)
}

fn decode_among<T: serde::de::DeserializeOwned>(
    areas: &'static HeldAreas,
    encoded: &'static [u8],
) -> T {
    let outer = DECODING.replace(Some(areas));
    let decoded = rmp_serde::from_slice(encoded).expect("the build script wrote this copy");
    DECODING.set(outer);
    decoded
}

/// Encode a method's body, and the bodies inside it, among the bodies that
/// run when the method is called.
pub(super) fn serialize_called<S: serde::Serializer>(
    body: &Body,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let outer = IN_A_METHOD.replace(true);
    let written = serde::Serialize::serialize(body, serializer);
    IN_A_METHOD.set(outer);
    written
}

impl Body {
    /// A body kept encoded until something reads it. Its bytes are never
    /// freed, so this is for code read once a run, such as the main
    /// script's methods.
    pub fn held(statements: Vec<Statement>) -> Self {
        let (opens_in_the_program, count) = (opens_in_the_program(&statements), count(&statements));
        // A copy of the bytes takes a block of their own length, where the
        // encoding grew into more.
        let bytes: Box<[u8]> = encode_held(&statements).as_slice().into();
        let (areas, outer) = held_areas(Box::leak(bytes));
        drop(statements);
        Body(Rc::new(Statements {
            decoded: OnceCell::new(),
            opens_in_the_program,
            count,
            encoded: Cell::new(Some((areas, outer))),
            shared_by_methods: OnceCell::new(),
        }))
    }

    /// Read the statements with `read`. A body still encoded is decoded for
    /// this read alone and its tree freed after, for what is read once,
    /// such as a method's body as it compiles.
    /// What every method defined with these statements shares, made by
    /// `make` the first time it is asked for.
    pub fn shared_by_methods<T: 'static>(&self, make: impl FnOnce() -> Rc<T>) -> Rc<T> {
        let held = self.0.shared_by_methods.get_or_init(|| make());
        Rc::clone(held)
            .downcast::<T>()
            .expect("the methods of a body share one kind of value")
    }

    pub fn read_once<R>(&self, read: impl FnOnce(&[Statement]) -> R) -> R {
        match (self.0.decoded.get(), self.0.encoded.get()) {
            (None, Some((areas, encoded))) => {
                let statements: Box<[Statement]> = {
                    let _counted = crate::memory_category::attribute(
                        crate::memory_category::Category::SyntaxTree,
                    );
                    decode_among(areas, encoded)
                };
                read(&statements)
            }
            _ => read(self),
        }
    }

    /// Whether there is a first statement and the program wrote it rather
    /// than the prelude, which a body still encoded answers without being
    /// decoded.
    pub fn opens_in_the_program(&self) -> bool {
        self.0.opens_in_the_program
    }

    /// How many statements there are, which a body still encoded answers
    /// without being decoded.
    pub fn len(&self) -> usize {
        self.0.count as usize
    }

    /// Whether there are no statements, which a body still encoded answers
    /// without being decoded.
    pub fn is_empty(&self) -> bool {
        self.0.count == 0
    }

    /// Whether the statements have been decoded and are kept.
    pub fn is_decoded(&self) -> bool {
        self.0.decoded.get().is_some()
    }

    fn decoded(statements: Box<[Statement]>) -> Self {
        Body(Rc::new(Statements {
            opens_in_the_program: opens_in_the_program(&statements),
            count: count(&statements),
            decoded: OnceCell::from(statements),
            encoded: Cell::new(None),
            shared_by_methods: OnceCell::new(),
        }))
    }
}

impl std::ops::Deref for Body {
    type Target = [Statement];

    fn deref(&self) -> &[Statement] {
        self.0.decoded.get_or_init(|| {
            let (areas, encoded) = self.0.encoded.take().expect("a body holds its statements");
            let _counted =
                crate::memory_category::attribute(crate::memory_category::Category::SyntaxTree);
            decode_among(areas, encoded)
        })
    }
}

impl From<Vec<Statement>> for Body {
    fn from(mut statements: Vec<Statement>) -> Self {
        crate::parser::tighten(&mut statements);
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

fn count(statements: &[Statement]) -> u32 {
    u32::try_from(statements.len()).expect("a body holds fewer than four billion statements")
}

fn opens_in_the_program(statements: &[Statement]) -> bool {
    statements
        .first()
        .is_some_and(|statement| !statement.position().prelude)
}

/// A body is written as which area its bytes are in, where they start
/// there, how many there are, whether the program wrote its first
/// statement, and how many statements it holds.
impl serde::Serialize for Body {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let encoded = rmp_serde::to_vec(&**self).map_err(serde::ser::Error::custom)?;
        let when_called = IN_A_METHOD.get();
        let start = ENCODING.with(|bodies| {
            let mut bodies = bodies.borrow_mut();
            let bodies = bodies
                .as_mut()
                .ok_or_else(|| serde::ser::Error::custom("a body encoded outside encode_held"))?;
            let area = if when_called {
                &mut bodies.when_called
            } else {
                &mut bodies.at_load
            };
            let start = area.len();
            area.extend(encoded.iter());
            Ok(start)
        })?;
        let place = |at: usize| u32::try_from(at).map_err(serde::ser::Error::custom);
        (
            when_called,
            place(start)?,
            place(encoded.len())?,
            self.opens_in_the_program(),
            self.0.count,
        )
            .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Body {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (when_called, start, length, opens_in_the_program, count) =
            <(bool, u32, u32, bool, u32)>::deserialize(deserializer)?;
        let areas = DECODING
            .get()
            .ok_or_else(|| serde::de::Error::custom("a body decoded outside decode_held"))?;
        let area = if when_called {
            areas.when_called
        } else {
            areas.at_load
        };
        let encoded = area
            .get(start as usize..start as usize + length as usize)
            .ok_or_else(|| serde::de::Error::custom("a body outside the copy being decoded"))?;
        Ok(Body(Rc::new(Statements {
            decoded: OnceCell::new(),
            opens_in_the_program,
            count,
            encoded: Cell::new(Some((areas, encoded))),
            shared_by_methods: OnceCell::new(),
        })))
    }
}
