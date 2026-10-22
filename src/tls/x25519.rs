// X25519, the Diffie-Hellman function over Curve25519 that RFC 7748
// defines, which TLS 1.3 agrees its keys with.

/// A number modulo 2^255 - 19, as five limbs of 51 bits each, least
/// significant first.
#[derive(Clone, Copy)]
struct Field([u64; 5]);

const MASK: u64 = (1 << 51) - 1;

impl Field {
    const ZERO: Field = Field([0; 5]);
    const ONE: Field = Field([1, 0, 0, 0, 0]);

    fn from_bytes(bytes: &[u8; 32]) -> Field {
        let word = |at: usize| {
            let mut held = [0u8; 8];
            held.copy_from_slice(&bytes[at..at + 8]);
            u64::from_le_bytes(held)
        };
        Field([
            word(0) & MASK,
            (word(6) >> 3) & MASK,
            (word(12) >> 6) & MASK,
            (word(19) >> 1) & MASK,
            (word(24) >> 12) & MASK,
        ])
    }

    fn to_bytes(self) -> [u8; 32] {
        let mut limbs = self.carried().0;
        // Subtract the modulus once if the value reaches it, which leaves
        // the one representative below it.
        let mut borrow = (limbs[0] + 19) >> 51;
        for limb in limbs.iter().skip(1) {
            borrow = (limb + borrow) >> 51;
        }
        limbs[0] += 19 * borrow;
        let mut carry = 0;
        for limb in limbs.iter_mut() {
            *limb += carry;
            carry = *limb >> 51;
            *limb &= MASK;
        }
        let mut written = [0u8; 32];
        let mut accumulated: u128 = 0;
        let mut bits = 0;
        let mut at = 0;
        for limb in limbs {
            accumulated |= (limb as u128) << bits;
            bits += 51;
            while bits >= 8 && at < 32 {
                written[at] = accumulated as u8;
                accumulated >>= 8;
                bits -= 8;
                at += 1;
            }
        }
        if at < 32 {
            written[at] = accumulated as u8;
        }
        written
    }

    fn carried(self) -> Field {
        let mut limbs = self.0;
        for _ in 0..2 {
            let mut carry = 0;
            for limb in limbs.iter_mut() {
                *limb += carry;
                carry = *limb >> 51;
                *limb &= MASK;
            }
            limbs[0] += carry * 19;
        }
        Field(limbs)
    }

    fn add(self, other: Field) -> Field {
        let mut limbs = [0; 5];
        for (at, limb) in limbs.iter_mut().enumerate() {
            *limb = self.0[at] + other.0[at];
        }
        Field(limbs).carried()
    }

    fn subtract(self, other: Field) -> Field {
        // Adding a multiple of the modulus first keeps every limb positive.
        const TWICE_MODULUS: [u64; 5] = [
            0xFFFFFFFFFFFDA,
            0xFFFFFFFFFFFFE,
            0xFFFFFFFFFFFFE,
            0xFFFFFFFFFFFFE,
            0xFFFFFFFFFFFFE,
        ];
        let mut limbs = [0; 5];
        for (at, limb) in limbs.iter_mut().enumerate() {
            *limb = self.0[at] + TWICE_MODULUS[at] - other.0[at];
        }
        Field(limbs).carried()
    }

    fn multiply(self, other: Field) -> Field {
        let left = self.0.map(u128::from);
        let right = other.0.map(u128::from);
        let folded: [u128; 5] = std::array::from_fn(|at| right[at] * 19);
        let mut wide = [0u128; 5];
        for (place, total) in wide.iter_mut().enumerate() {
            for (index, limb) in left.iter().enumerate() {
                let partner = (place + 5 - index) % 5;
                let factor = if partner > place {
                    folded[partner]
                } else {
                    right[partner]
                };
                *total += limb * factor;
            }
        }
        let mut limbs = [0u64; 5];
        let mut carry: u128 = 0;
        for (at, total) in wide.iter().enumerate() {
            let held = total + carry;
            limbs[at] = (held as u64) & MASK;
            carry = held >> 51;
        }
        limbs[0] += (carry as u64) * 19;
        Field(limbs).carried()
    }

    fn square(self) -> Field {
        self.multiply(self)
    }

    fn times_small(self, factor: u64) -> Field {
        let mut limbs = [0u64; 5];
        let mut carry: u128 = 0;
        for (at, limb) in limbs.iter_mut().enumerate() {
            let held = self.0[at] as u128 * factor as u128 + carry;
            *limb = (held as u64) & MASK;
            carry = held >> 51;
        }
        limbs[0] += (carry as u64) * 19;
        Field(limbs).carried()
    }

    /// The inverse, as the value raised to the modulus less two.
    fn invert(self) -> Field {
        let mut result = Field::ONE;
        // 2^255 - 21, bit by bit from the top.
        let exponent: [u8; 32] = {
            let mut bytes = [0xFFu8; 32];
            bytes[0] = 0xEB;
            bytes[31] = 0x7F;
            bytes
        };
        for bit in (0..255).rev() {
            result = result.square();
            if (exponent[bit / 8] >> (bit % 8)) & 1 == 1 {
                result = result.multiply(self);
            }
        }
        result
    }

    /// Swap the two values when `swap` is 1, without branching on it.
    fn conditional_swap(first: &mut Field, second: &mut Field, swap: u64) {
        let mask = 0u64.wrapping_sub(swap);
        for at in 0..5 {
            let flipped = mask & (first.0[at] ^ second.0[at]);
            first.0[at] ^= flipped;
            second.0[at] ^= flipped;
        }
    }
}

/// The shared secret the scalar and the point give, both as 32 bytes.
pub fn x25519(scalar: &[u8; 32], point: &[u8; 32]) -> [u8; 32] {
    let mut clamped = *scalar;
    clamped[0] &= 248;
    clamped[31] &= 127;
    clamped[31] |= 64;
    let mut masked_point = *point;
    masked_point[31] &= 127;
    let base = Field::from_bytes(&masked_point);
    let (mut x2, mut z2) = (Field::ONE, Field::ZERO);
    let (mut x3, mut z3) = (base, Field::ONE);
    let mut swap = 0u64;
    for bit in (0..255).rev() {
        let current = ((clamped[bit / 8] >> (bit % 8)) & 1) as u64;
        swap ^= current;
        Field::conditional_swap(&mut x2, &mut x3, swap);
        Field::conditional_swap(&mut z2, &mut z3, swap);
        swap = current;
        let a = x2.add(z2);
        let aa = a.square();
        let b = x2.subtract(z2);
        let bb = b.square();
        let e = aa.subtract(bb);
        let c = x3.add(z3);
        let d = x3.subtract(z3);
        let da = d.multiply(a);
        let cb = c.multiply(b);
        x3 = da.add(cb).square();
        z3 = base.multiply(da.subtract(cb).square());
        x2 = aa.multiply(bb);
        z2 = e.multiply(aa.add(e.times_small(121665)));
    }
    Field::conditional_swap(&mut x2, &mut x3, swap);
    Field::conditional_swap(&mut z2, &mut z3, swap);
    x2.multiply(z2.invert()).to_bytes()
}

/// The public key a private scalar stands for: the scalar times the base
/// point, whose x coordinate is 9.
pub fn public_key(scalar: &[u8; 32]) -> [u8; 32] {
    let mut base = [0u8; 32];
    base[0] = 9;
    x25519(scalar, &base)
}
