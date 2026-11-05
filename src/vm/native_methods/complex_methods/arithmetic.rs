// The arithmetic a Complex answers, part by part.

use super::*;
use crate::object::View;

impl VirtualMachine {
    /// `+`, `-`, `*`, and `/` between a Complex and whatever it was handed,
    /// composed from the four parts.
    pub(crate) fn complex_arithmetic(
        &mut self,
        real: &Object,
        imaginary: &Object,
        method_name: &str,
        other: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // A real operand reaches each part on its own, as MRI's
        // `rb_complex_mul` and `f_divide` apply it.
        if complex_parts(other).is_none() && method_name != "+" && method_name != "-" {
            let (new_real, new_imaginary) = if method_name == "*" {
                (
                    self.multiply_part_by(real, other, position)?,
                    self.multiply_part_by(imaginary, other, position)?,
                )
            } else {
                let new_real = self.quotient_part(real, other, position)?;
                let new_imaginary = self.quotient_part(imaginary, other, position)?;
                (
                    canonical_rational(new_real),
                    canonical_rational(new_imaginary),
                )
            };
            return self.make_complex(new_real, new_imaginary, position);
        }
        let (other_real, other_imaginary) = self.complex_operand_parts(other, position)?;
        match method_name {
            "+" | "-" => {
                let combine = |vm: &mut Self, left: &Object, right: &Object| {
                    if method_name == "+" {
                        vm.add_parts(left, right, position)
                    } else {
                        vm.subtract_parts(left, right, position)
                    }
                };
                let new_real = combine(self, real, &other_real)?;
                let new_imaginary = combine(self, imaginary, &other_imaginary)?;
                self.make_complex(new_real, new_imaginary, position)
            }
            // (a + bi)(c + di) = (ac - bd) + (ad + bc)i
            "*" => {
                let ac = self.multiply_parts(real, &other_real, position)?;
                let bd = self.multiply_parts(imaginary, &other_imaginary, position)?;
                let ad = self.multiply_parts(real, &other_imaginary, position)?;
                let bc = self.multiply_parts(imaginary, &other_real, position)?;
                let new_real = self.subtract_parts(&ac, &bd, position)?;
                let new_imaginary = self.add_parts(&ad, &bc, position)?;
                self.make_complex(new_real, new_imaginary, position)
            }
            // Dividing multiplies by the conjugate of the divisor, which
            // leaves a real denominator to divide each part by.
            _ => {
                // A divisor that stands on the real line divides each part on
                // its own, so dividing by zero answers infinities rather than
                // the NaN the conjugate form would give.
                if matches!(other_imaginary.view(), View::Int(0)) {
                    let new_real = self.divide_parts(real, &other_real, position)?;
                    let new_imaginary = self.divide_parts(imaginary, &other_real, position)?;
                    return self.make_complex(new_real, new_imaginary, position);
                }
                let cc = self.multiply_parts(&other_real, &other_real, position)?;
                let dd = self.multiply_parts(&other_imaginary, &other_imaginary, position)?;
                let denominator = self.add_parts(&cc, &dd, position)?;
                let ac = self.multiply_parts(real, &other_real, position)?;
                let bd = self.multiply_parts(imaginary, &other_imaginary, position)?;
                let bc = self.multiply_parts(imaginary, &other_real, position)?;
                let ad = self.multiply_parts(real, &other_imaginary, position)?;
                let real_top = self.add_parts(&ac, &bd, position)?;
                let imaginary_top = self.subtract_parts(&bc, &ad, position)?;
                let new_real = self.divide_parts(&real_top, &denominator, position)?;
                let new_imaginary = self.divide_parts(&imaginary_top, &denominator, position)?;
                self.make_complex(new_real, new_imaginary, position)
            }
        }
    }

    /// A Complex raised to a whole power, by multiplying it out. A negative
    /// exponent divides one by the positive power instead.
    pub(crate) fn complex_power(
        &mut self,
        real: &Object,
        imaginary: &Object,
        exponent: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let View::Int(exponent) = exponent.view() else {
            // Anything that is not a whole power is taken in polar form,
            // where `z ** w` is `exp(w * log z)`.
            if complex_parts(exponent).is_some() || self.is_real_operand(exponent) {
                return self.complex_polar_power(real, imaginary, exponent, position);
            }
            // A power that is none of the numbers coerces the pair, and the
            // two it answers are raised in its own terms.
            let refused = format!(
                "{} can't be coerced into Complex",
                crate::vm::errors::coercion_subject(exponent)
            );
            if !self.responds_to(exponent, "coerce") {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &refused,
                    position,
                ));
            }
            let base = self.make_complex(real.clone(), imaginary.clone(), position)?;
            let coerced = self.send_to_object(exponent.clone(), "coerce", vec![base], position)?;
            let View::Array(pair) = coerced.view() else {
                let message = refused;
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &message,
                    position,
                ));
            };
            let pair = pair.borrow().clone();
            let [left, right] = pair.as_slice() else {
                let message = "coerce must return [x, y]".to_string();
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &message,
                    position,
                ));
            };
            return self.send_to_object(left.clone(), "**", vec![right.clone()], position);
        };
        let mut answer = self.make_complex(Object::Int(1), Object::Int(0), position)?;
        let base = self.make_complex(real.clone(), imaginary.clone(), position)?;
        for _ in 0..exponent.unsigned_abs() {
            answer = self.send_to_object(answer, "*", vec![base.clone()], position)?;
        }
        if exponent < 0 {
            let one = self.make_complex(Object::Int(1), Object::Int(0), position)?;
            return self.send_to_object(one, "/", vec![answer], position);
        }
        Ok(answer)
    }

    /// A Complex raised to a power that is not a whole number, worked in
    /// polar form: `z ** w` is `exp(w * log z)`.
    pub(crate) fn complex_polar_power(
        &mut self,
        real: &Object,
        imaginary: &Object,
        exponent: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let base_real = self.float_value_of(real, position)?;
        let base_imaginary = self.float_value_of(imaginary, position)?;
        let (power_real, power_imaginary) = self.complex_operand_parts(exponent, position)?;
        let power_real = self.part_to_float(&power_real, position)?;
        let power_imaginary = self.part_to_float(&power_imaginary, position)?;
        let power_real = self.float_value_of(&power_real, position)?;
        let power_imaginary = self.float_value_of(&power_imaginary, position)?;
        let length = base_real.hypot(base_imaginary);
        let angle = base_imaginary.atan2(base_real);
        let log_length = length.ln();
        let grown = (power_real * log_length - power_imaginary * angle).exp();
        let turned = power_real * angle + power_imaginary * log_length;
        self.make_complex(
            Object::Float(grown * turned.cos()),
            Object::Float(grown * turned.sin()),
            position,
        )
    }

    /// The two parts of an operand: a Complex gives both, and a real number
    /// gives itself with nothing on the imaginary axis.
    pub(crate) fn complex_operand_parts(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<(Object, Object), MetorexError> {
        if let Some(parts) = complex_parts(value) {
            return Ok(parts);
        }
        if self.is_real_operand(value) {
            return Ok((value.clone(), Object::Int(0)));
        }
        let message = format!(
            "{} can't be coerced into Complex",
            crate::vm::errors::coercion_subject(value)
        );
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }

    pub(crate) fn add_parts(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Add,
            left.clone(),
            right.clone(),
            position,
        )
    }

    pub(crate) fn subtract_parts(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Subtract,
            left.clone(),
            right.clone(),
            position,
        )
    }

    /// A part times a real number, with the shortcuts MRI's `f_mul` takes:
    /// an Integer zero on either side of an Integer product is zero, and a
    /// factor of one leaves the other side as it is.
    fn multiply_part_by(
        &mut self,
        part: &Object,
        factor: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let integer = |held: &Object| matches!(held.view(), View::Int(_) | View::BigInt(_));
        let is_int =
            |held: &Object, wanted: i64| matches!(held.view(), View::Int(n) if n == wanted);
        if integer(part) {
            if is_int(factor, 0) || (is_int(part, 0) && integer(factor)) {
                return Ok(Object::Int(0));
            }
            if is_int(part, 1) {
                return Ok(factor.clone());
            }
        }
        if is_int(factor, 1) {
            return Ok(part.clone());
        }
        self.multiply_parts(part, factor, position)
    }

    /// A part divided by a real number as MRI's `f_quo` divides it: a Float
    /// or a Rational by its division, and anything else by its `quo`.
    fn quotient_part(
        &mut self,
        part: &Object,
        divisor: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if matches!(part.view(), View::Float(_))
            || crate::vm::native_methods::rational_methods::rational_parts(part).is_some()
        {
            return self.evaluate_binary_operation(
                &crate::ast::BinaryOp::Divide,
                part.clone(),
                divisor.clone(),
                position,
            );
        }
        self.send_to_object(part.clone(), "quo", vec![divisor.clone()], position)
    }

    pub(crate) fn multiply_parts(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Multiply,
            left.clone(),
            right.clone(),
            position,
        )
    }

    /// Dividing two parts keeps them exact: two Integers answer the Rational
    /// between them rather than the whole number the division would floor to.
    pub(crate) fn divide_parts(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if matches!(left.view(), View::Int(_) | View::BigInt(_))
            && matches!(right.view(), View::Int(_) | View::BigInt(_))
        {
            return self.send_to_object(left.clone(), "quo", vec![right.clone()], position);
        }
        self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Divide,
            left.clone(),
            right.clone(),
            position,
        )
    }

    pub(crate) fn negate_part(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.subtract_parts(&Object::Int(0), value, position)
    }

    /// A part as a Float, which is what the magnitude and the angle measure.
    pub(crate) fn part_to_float(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.send_to_object(value.clone(), "to_f", vec![], position)
    }
}

/// A Rational whose denominator is one as the Integer it stands for, and
/// anything else as it is.
fn canonical_rational(value: Object) -> Object {
    match crate::vm::native_methods::rational_methods::rational_parts(&value) {
        Some((numerator, denominator)) if denominator == num_bigint::BigInt::from(1) => {
            Object::integer(numerator)
        }
        _ => value,
    }
}
