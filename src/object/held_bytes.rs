//! The bytes of a String, kept inside the String when they are few enough.

/// The most bytes kept inside the String rather than in an allocation of
/// their own. With the length beside them they take what a `Vec` does.
pub(crate) const HELD_IN_PLACE: usize = 0;

#[derive(Debug, Clone)]
pub(crate) enum HeldBytes {
    InPlace {
        length: u8,
        bytes: [u8; HELD_IN_PLACE],
    },
    Allocated(Vec<u8>),
}

impl From<Vec<u8>> for HeldBytes {
    fn from(bytes: Vec<u8>) -> Self {
        if bytes.len() > HELD_IN_PLACE {
            return HeldBytes::Allocated(bytes);
        }
        let mut held = [0; HELD_IN_PLACE];
        held[..bytes.len()].copy_from_slice(&bytes);
        HeldBytes::InPlace {
            length: bytes.len() as u8,
            bytes: held,
        }
    }
}

impl HeldBytes {
    /// The bytes allocated for them, which is none while they are in place.
    pub(crate) fn capacity(&self) -> usize {
        match self {
            HeldBytes::InPlace { .. } => 0,
            HeldBytes::Allocated(bytes) => bytes.capacity(),
        }
    }

    pub(crate) fn extend_from_slice(&mut self, extra: &[u8]) {
        match self {
            HeldBytes::InPlace { length, bytes } => {
                let start = *length as usize;
                let end = start + extra.len();
                if end <= HELD_IN_PLACE {
                    bytes[start..end].copy_from_slice(extra);
                    *length = end as u8;
                    return;
                }
                // A string that outgrows its place is likely to keep growing,
                // so it is given room for twice what it holds.
                let mut allocated = Vec::with_capacity(end * 2);
                allocated.extend_from_slice(&bytes[..start]);
                allocated.extend_from_slice(extra);
                *self = HeldBytes::Allocated(allocated);
            }
            HeldBytes::Allocated(bytes) => bytes.extend_from_slice(extra),
        }
    }
}

impl std::ops::Deref for HeldBytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match self {
            HeldBytes::InPlace { length, bytes } => &bytes[..*length as usize],
            HeldBytes::Allocated(bytes) => bytes,
        }
    }
}

impl std::ops::DerefMut for HeldBytes {
    fn deref_mut(&mut self) -> &mut [u8] {
        match self {
            HeldBytes::InPlace { length, bytes } => &mut bytes[..*length as usize],
            HeldBytes::Allocated(bytes) => bytes,
        }
    }
}
