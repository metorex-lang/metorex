//! The errors a `rescue` can catch.

use crate::error::MetorexError;
use crate::object::Object;

/// `error` as an exception Ruby code can rescue: the interpreter's own
/// RuntimeError and TypeError become the exceptions of those classes, and
/// every other error stays as it is.
pub(crate) fn as_rescuable(error: MetorexError) -> MetorexError {
    let (class_name, message, location) = match error {
        MetorexError::RuntimeError {
            message, location, ..
        } => ("RuntimeError", message, location),
        MetorexError::TypeError {
            message, location, ..
        } => ("TypeError", message, location),
        other => return other,
    };
    MetorexError::UncaughtException {
        exception: Object::exception(class_name, message.clone()),
        location,
        message,
    }
}
