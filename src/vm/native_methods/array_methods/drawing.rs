// Drawing elements at random.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// The count and the `random:` source `Array#sample` was called with.
    pub(crate) fn sample_arguments(
        &mut self,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<(Option<i64>, Option<Object>), MetorexError> {
        let mut positional = arguments;
        let mut source = None;
        if let Some(View::Dict(entries)) = arguments.last().view() {
            let named = entries.borrow().get(":random").cloned();
            if let Some(named) = named {
                source = Some(named);
                positional = &arguments[..arguments.len() - 1];
            }
        }
        if positional.len() > 1 {
            return Err(method_argument_error(
                method_name,
                1,
                positional.len(),
                position,
            ));
        }
        let count = match positional.first().view() {
            None => None,
            Some(View::Int(count)) => Some(count),
            Some(other) => {
                let counted = self.coerce_integer_argument(&other.to_object(), position)?;
                Some(counted.try_into().unwrap_or(i64::MAX))
            }
        };
        Ok((count, source))
    }

    /// An index below `limit`, drawn either from the interpreter's own
    /// generator or from the object a `random:` keyword named. Ruby asks such
    /// an object for `rand(limit)` and refuses an answer outside the range.
    pub(crate) fn random_upto(
        &mut self,
        source: Option<&Object>,
        limit: i64,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let Some(source) = source else {
            return Ok(self.next_random_int(limit));
        };
        if limit <= 0 {
            return Ok(0);
        }
        let drawn =
            self.send_to_object(source.clone(), "rand", vec![Object::Int(limit)], position)?;
        let index = match drawn.view() {
            View::Int(index) => index,
            View::Float(index) => index.trunc() as i64,
            _ => {
                let drawn = self.coerce_integer_argument(&drawn, position)?;
                drawn.try_into().unwrap_or(i64::MAX)
            }
        };
        if index < 0 || index >= limit {
            return Err(simple_exception(
                "RangeError",
                &format!("random number too big {index}"),
                position,
            ));
        }
        Ok(index)
    }
}
