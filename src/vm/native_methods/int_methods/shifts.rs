// How far a shift moves.

use super::*;
use crate::object::View;

impl VirtualMachine {
    pub(crate) fn shift_count(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: crate::lexer::Position,
    ) -> Result<ShiftCount, crate::error::MetorexError> {
        let held = match argument.view() {
            View::Int(_) | View::BigInt(_) => argument.clone(),
            _ if self.responds_to(argument, "to_int") => (match self
                .send_to_object(argument.clone(), "to_int", vec![], position)?
                .view()
            {
                answered @ (View::Int(_) | View::BigInt(_)) => answered,
                _ => {
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        argument,
                        position,
                    ));
                }
            })
            .to_object(),
            _ => {
                return Err(method_argument_type_error(
                    method_name,
                    "Integer",
                    argument,
                    position,
                ));
            }
        };
        let negative = matches!(held.view(), View::Int(count) if count < 0)
            || matches!(held.view(), View::BigInt(count) if **count < num_bigint::BigInt::from(0));
        let written = match held.view() {
            View::Int(count) => Some(count),
            View::BigInt(count) => count.to_string().parse::<i64>().ok(),
            _ => None,
        };
        Ok(ShiftCount { written, negative })
    }
}

/// How far a shift moves, read the way Ruby reads it: an Integer outright, or
/// anything that answers `to_int` with one. A count too wide for a machine
/// word has no place to name, and the sign says which way it went.
pub(crate) struct ShiftCount {
    pub written: Option<i64>,
    pub negative: bool,
}
