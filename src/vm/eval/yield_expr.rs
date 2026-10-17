// `yield` expression: invoke the block bound to the current method.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::object::{OptionView, View};

use crate::vm::core::VirtualMachine;
use crate::vm::utils::position_to_location;

impl VirtualMachine {
    /// The block the running method was called with, which `yield` calls.
    pub(crate) fn block_to_yield(&mut self, position: Position) -> Result<Object, MetorexError> {
        let block = self.environment().get("__block__").or_else(|| {
            // Also check if there's a named block parameter
            self.environment().get("block_given?").and_then(|bg| {
                if bg == Object::Bool(true) {
                    // The block was bound to a named parameter — find it
                    None
                } else {
                    None
                }
            })
        });

        match block.view() {
            Some(View::Block(_)) => Ok(block.expect("a block was found")),
            _ => {
                // Ruby names this a jump with nowhere to land rather than a
                // plain runtime failure.
                let message = "no block given (yield)".to_string();
                Err(MetorexError::UncaughtException {
                    exception: crate::object::Object::exception("LocalJumpError", message.clone()),
                    location: position_to_location(position),
                    message,
                })
            }
        }
    }
}
