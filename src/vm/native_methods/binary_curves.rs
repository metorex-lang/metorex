// Points on an elliptic curve over a binary field GF(2^m), y^2 + xy = x^3 +
// ax^2 + b, for OpenSSL::PKey::EC's binary curves. A field element is a
// polynomial over GF(2) held as little-endian 64-bit limbs, reduced by the
// field's polynomial, and a point is held by its affine coordinates.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::object::{OptionView, View};
use crate::vm::core::VirtualMachine;

type Element = Vec<u64>;

/// GF(2^m), by the degree m and the polynomial it reduces by.
struct Field {
    degree: usize,
    modulus: Element,
    limbs: usize,
}

fn bit(value: &[u64], at: usize) -> bool {
    value
        .get(at / 64)
        .is_some_and(|limb| limb >> (at % 64) & 1 == 1)
}

fn highest_bit(value: &[u64]) -> Option<usize> {
    value
        .iter()
        .enumerate()
        .rev()
        .find(|(_, limb)| **limb != 0)
        .map(|(at, limb)| at * 64 + 63 - limb.leading_zeros() as usize)
}

fn xor_into(target: &mut [u64], source: &[u64]) {
    for (held, given) in target.iter_mut().zip(source) {
        *held ^= given;
    }
}

/// `value` shifted up `count` bits, in `limbs` limbs.
fn shifted(value: &[u64], count: usize, limbs: usize) -> Element {
    let mut answer = vec![0u64; limbs];
    let (whole, part) = (count / 64, count % 64);
    for (at, limb) in value.iter().enumerate() {
        if at + whole < limbs {
            answer[at + whole] ^= limb << part;
        }
        if part != 0 && at + whole + 1 < limbs {
            answer[at + whole + 1] ^= limb >> (64 - part);
        }
    }
    answer
}

impl Field {
    /// The field the exponents of its polynomial's terms name, highest first.
    fn new(terms: &[usize]) -> Option<Field> {
        let degree = *terms.first()?;
        let limbs = degree / 64 + 2;
        let mut modulus = vec![0u64; limbs];
        for term in terms {
            modulus[term / 64] |= 1 << (term % 64);
        }
        Some(Field {
            degree,
            modulus,
            limbs,
        })
    }

    fn zero(&self) -> Element {
        vec![0u64; self.limbs]
    }

    fn one(&self) -> Element {
        let mut answer = self.zero();
        answer[0] = 1;
        answer
    }

    fn element_from_hex(&self, text: &str) -> Option<Element> {
        let mut answer = self.zero();
        for (at, digit) in text.bytes().rev().enumerate() {
            let value = (digit as char).to_digit(16)? as u64;
            if at / 16 >= self.limbs {
                if value != 0 {
                    return None;
                }
                continue;
            }
            answer[at / 16] |= value << ((at % 16) * 4);
        }
        Some(answer)
    }

    fn to_hex(&self, value: &[u64]) -> String {
        let text: String = value
            .iter()
            .rev()
            .map(|limb| format!("{limb:016x}"))
            .collect();
        let trimmed = text.trim_start_matches('0');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            trimmed.to_string()
        }
    }

    fn add(&self, first: &[u64], second: &[u64]) -> Element {
        let mut answer = first.to_vec();
        xor_into(&mut answer, second);
        answer
    }

    /// The product, by shifting and adding over the bits of `second` from
    /// the top, reducing whenever the running total reaches degree m.
    fn multiply(&self, first: &[u64], second: &[u64]) -> Element {
        let mut total = self.zero();
        let Some(top) = highest_bit(second) else {
            return total;
        };
        for at in (0..=top).rev() {
            total = shifted(&total, 1, self.limbs);
            if bit(&total, self.degree) {
                xor_into(&mut total, &self.modulus);
            }
            if bit(second, at) {
                xor_into(&mut total, first);
            }
        }
        total
    }

    fn square(&self, value: &[u64]) -> Element {
        self.multiply(value, value)
    }

    /// The inverse, by the extended Euclidean algorithm over polynomials.
    fn invert(&self, value: &[u64]) -> Option<Element> {
        highest_bit(value)?;
        let mut upper = value.to_vec();
        let mut lower = self.modulus.clone();
        let mut first = self.one();
        let mut second = self.zero();
        while highest_bit(&upper) != Some(0) {
            let upper_degree = highest_bit(&upper)?;
            let lower_degree = highest_bit(&lower)?;
            if upper_degree < lower_degree {
                std::mem::swap(&mut upper, &mut lower);
                std::mem::swap(&mut first, &mut second);
                continue;
            }
            let gap = upper_degree - lower_degree;
            xor_into(&mut upper, &shifted(&lower, gap, self.limbs));
            xor_into(&mut first, &shifted(&second, gap, self.limbs));
        }
        Some(first)
    }

    fn divide(&self, top: &[u64], bottom: &[u64]) -> Option<Element> {
        Some(self.multiply(top, &self.invert(bottom)?))
    }

    /// The half-trace, which solves z^2 + z = c when m is odd.
    fn half_trace(&self, value: &[u64]) -> Element {
        let mut answer = value.to_vec();
        for _ in 0..(self.degree - 1) / 2 {
            answer = self.square(&self.square(&answer));
            xor_into(&mut answer, value);
        }
        answer
    }
}

type Point = Option<(Element, Element)>;

struct Curve {
    field: Field,
    a: Element,
    b: Element,
}

impl Curve {
    fn on_curve(&self, x: &[u64], y: &[u64]) -> bool {
        let field = &self.field;
        let left = field.add(&field.square(y), &field.multiply(x, y));
        let x_squared = field.square(x);
        let right = field.add(
            &field.add(
                &field.multiply(&x_squared, x),
                &field.multiply(&self.a, &x_squared),
            ),
            &self.b,
        );
        left == right
    }

    fn double(&self, point: &Point) -> Point {
        let (x, y) = point.as_ref()?;
        let field = &self.field;
        highest_bit(x)?;
        let slope = field.add(x, &field.divide(y, x)?);
        let new_x = field.add(&field.add(&field.square(&slope), &slope), &self.a);
        let new_y = field.add(
            &field.square(x),
            &field.multiply(&field.add(&slope, &field.one()), &new_x),
        );
        Some((new_x, new_y))
    }

    fn add(&self, first: &Point, second: &Point) -> Point {
        let Some((x1, y1)) = first else {
            return second.clone();
        };
        let Some((x2, y2)) = second else {
            return first.clone();
        };
        let field = &self.field;
        if x1 == x2 {
            return if y1 == y2 { self.double(first) } else { None };
        }
        let slope = field.divide(&field.add(y1, y2), &field.add(x1, x2))?;
        let new_x = field.add(
            &field.add(
                &field.add(&field.square(&slope), &slope),
                &field.add(x1, x2),
            ),
            &self.a,
        );
        let new_y = field.add(
            &field.add(&field.multiply(&slope, &field.add(x1, &new_x)), &new_x),
            y1,
        );
        Some((new_x, new_y))
    }

    /// The point times the scalar whose big-endian bytes are given.
    fn multiply(&self, point: &Point, scalar: &[u8]) -> Point {
        let mut total: Point = None;
        for byte in scalar {
            for at in (0..8).rev() {
                total = self.double(&total);
                if byte >> at & 1 == 1 {
                    total = self.add(&total, point);
                }
            }
        }
        total
    }
}

fn text_of(object: Option<&Object>) -> Option<String> {
    match object.view() {
        Some(View::String(held)) => Some(held.as_str().to_string()),
        _ => None,
    }
}

impl VirtualMachine {
    /// `OpenSSL.__ec2m__(operation, terms, a, b, ...)`: an operation on
    /// points of the binary curve given by its field polynomial's terms and
    /// its coefficients, every value in hex. A point at infinity is nil.
    ///
    /// - `"multiply", x, y, scalar` answers the point times the scalar.
    /// - `"add", x1, y1, x2, y2` answers the sum.
    /// - `"on_curve", x, y` answers whether the point lies on the curve.
    /// - `"y_bit", x, y` answers the bit SEC 1's compressed form keeps.
    /// - `"decompress", x, bit` answers y, or nil when there is none.
    pub(crate) fn binary_curve(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let refuse = || {
            MetorexError::runtime_error(
                "a binary curve operation names its field, coefficients, and points".to_string(),
                crate::vm::utils::position_to_location(position),
            )
        };
        let operation = text_of(arguments.first()).ok_or_else(refuse)?;
        let terms: Vec<usize> = match arguments.get(1).view() {
            Some(View::Array(held)) => held
                .borrow()
                .iter()
                .filter_map(|term| match term.view() {
                    View::Int(value) => usize::try_from(value).ok(),
                    _ => None,
                })
                .collect(),
            _ => return Err(refuse()),
        };
        let field = Field::new(&terms).ok_or_else(refuse)?;
        let coefficient =
            |at: usize| text_of(arguments.get(at)).and_then(|text| field.element_from_hex(&text));
        let (a, b) = (
            coefficient(2).ok_or_else(refuse)?,
            coefficient(3).ok_or_else(refuse)?,
        );
        let curve = Curve { field, a, b };
        let element = |at: usize| {
            text_of(arguments.get(at)).and_then(|text| curve.field.element_from_hex(&text))
        };
        let point_at = |at: usize| -> Result<Point, MetorexError> {
            if arguments.get(at).is_none_or(Object::is_nil) {
                return Ok(None);
            }
            Ok(Some((
                element(at).ok_or_else(refuse)?,
                element(at + 1).ok_or_else(refuse)?,
            )))
        };
        let answer = |curve: &Curve, point: Point| match point {
            Some((x, y)) => Object::array(vec![
                Object::string(curve.field.to_hex(&x)),
                Object::string(curve.field.to_hex(&y)),
            ]),
            None => Object::Nil,
        };
        match operation.as_str() {
            "multiply" => {
                let point = point_at(4)?;
                let scalar_hex = text_of(arguments.get(6)).ok_or_else(refuse)?;
                let padded = if scalar_hex.len() % 2 == 1 {
                    format!("0{scalar_hex}")
                } else {
                    scalar_hex
                };
                let scalar: Vec<u8> = (0..padded.len())
                    .step_by(2)
                    .map(|at| u8::from_str_radix(&padded[at..at + 2], 16))
                    .collect::<Result<_, _>>()
                    .map_err(|_| refuse())?;
                Ok(answer(&curve, curve.multiply(&point, &scalar)))
            }
            "add" => {
                let (first, second) = (point_at(4)?, point_at(6)?);
                Ok(answer(&curve, curve.add(&first, &second)))
            }
            "on_curve" => {
                let (x, y) = point_at(4)?.ok_or_else(refuse)?;
                Ok(Object::Bool(curve.on_curve(&x, &y)))
            }
            "y_bit" => {
                let (x, y) = point_at(4)?.ok_or_else(refuse)?;
                Ok(Object::Int(match curve.field.divide(&y, &x) {
                    Some(quotient) => (quotient[0] & 1) as i64,
                    None => 0,
                }))
            }
            "decompress" => {
                let x = element(4).ok_or_else(refuse)?;
                let wanted = matches!(arguments.get(5).view(), Some(View::Int(1)));
                let field = &curve.field;
                if highest_bit(&x).is_none() {
                    // x = 0 has the one y whose square is b.
                    let mut root = curve.b.clone();
                    for _ in 0..field.degree - 1 {
                        root = field.square(&root);
                    }
                    return Ok(Object::string(field.to_hex(&root)));
                }
                let inverse_squared = field.square(&field.invert(&x).ok_or_else(refuse)?);
                let target = field.add(
                    &field.add(&x, &curve.a),
                    &field.multiply(&curve.b, &inverse_squared),
                );
                let mut solution = field.half_trace(&target);
                if field.add(&field.square(&solution), &solution) != target {
                    return Ok(Object::Nil);
                }
                if (solution[0] & 1 == 1) != wanted {
                    xor_into(&mut solution, &field.one());
                }
                Ok(Object::string(field.to_hex(&field.multiply(&x, &solution))))
            }
            _ => Err(refuse()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const B571: [usize; 5] = [571, 10, 5, 2, 0];
    const B571_B: &str = "2f40e7e2221f295de297117b7f3d62f5c6a97ffcb8ceff1cd6ba8ce4a9a18ad84ffabbd8efa59332be7ad6756a66e294afd185a78ff12aa520e4de739baca0c7ffeff7f2955727a";
    // 2^553 G and 2^554 G on sect571r1, as OpenSSL computes them.
    const X553: &str = "1156a682ef64a1522a066b63389f0fadae4044b9f3fe5b4307439c2867cfdcbc1ccd6ffd9ac2eba59cbb103571cbcf4fa1ac80ea3da683d146543225ff9685df780b0c7a7816a97";
    const Y553: &str = "434fc638e047abcdb92582ab47b8b4e41ce110f08248e460eaa8e72ca80cf48a312db80e5143d4513e5bce24d798e13b89ebf1cf14fdc431791880ddf13d8d06d5f522c6a8263ed";
    const X554: &str = "a1cc99a2250bc566dfd190c12a101d97832e3ba8f2692b6ce51fb5c68d8604bf92c146a02e9461054ce0378d2cdf8afd3f201d23575c867b04465a6ec6285807a1d672b9631511";
    const Y554: &str = "4707c15da62c0aecfa76e70d92c8a473b6f4bb0a671323b34bdd0fc80190547b8a5f7b5ac1d6f6169271b87770aa100e2c271db5f6696bc400166fc3bb5a81df3b6d8c4749427c0";

    fn curve() -> Curve {
        let field = Field::new(&B571).unwrap();
        let a = field.one();
        let b = field.element_from_hex(B571_B).unwrap();
        Curve { field, a, b }
    }

    #[test]
    fn an_element_times_its_inverse_is_one() {
        let curve = curve();
        let field = &curve.field;
        for text in [X553, Y553, X554, B571_B, "1", "2", "3"] {
            let value = field.element_from_hex(text).unwrap();
            let inverse = field.invert(&value).unwrap();
            assert_eq!(field.multiply(&value, &inverse), field.one(), "{text}");
        }
    }

    #[test]
    fn doubling_a_point_matches_openssl() {
        let curve = curve();
        let field = &curve.field;
        let point = Some((
            field.element_from_hex(X553).unwrap(),
            field.element_from_hex(Y553).unwrap(),
        ));
        assert!(curve.on_curve(&point.as_ref().unwrap().0, &point.as_ref().unwrap().1));
        let doubled = curve.double(&point).unwrap();
        assert_eq!(field.to_hex(&doubled.0), X554);
        assert_eq!(field.to_hex(&doubled.1), Y554);
    }

    const GX: &str = "303001d34b856296c16c0d40d3cd7750a93d1d2955fa80aa5f40fc8db7b2abdbde53950f4c0d293cdd711a35b67fb1499ae60038614f1394abfa3b4c850d927e1e7769c8eec2d19";
    const GY: &str = "37bf27342da639b6dccfffeb73d69d78c6c27a6009cbbca1980f8533921e8a684423e43bab08a576291af8f461bb2a8b3531d2f0485c19b16e2f1516e23dd3c1a4827af1b8ac15b";

    #[test]
    fn multiplying_the_generator_matches_openssl() {
        let curve = curve();
        let field = &curve.field;
        let generator = Some((
            field.element_from_hex(GX).unwrap(),
            field.element_from_hex(GY).unwrap(),
        ));
        let mut scalar = vec![0u8; 70];
        scalar[0] = 0x04;
        let product = curve.multiply(&generator, &scalar).unwrap();
        assert_eq!(field.to_hex(&product.0), X554);
        assert_eq!(field.to_hex(&product.1), Y554);
    }
}
