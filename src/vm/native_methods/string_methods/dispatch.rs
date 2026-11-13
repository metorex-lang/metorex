// The order a String method is looked for in. Each step answers `None` when
// the name is none of its own, and the next step is tried.

use super::*;
use crate::object::View;

/// One step of the chain: every group of String methods reads the same
/// arguments and answers the same way.
type StringMethodStep = fn(
    &mut VirtualMachine,
    &Object,
    &Rc<crate::object::StringValue>,
    &str,
    &[Object],
    Position,
) -> Result<Option<Object>, MetorexError>;

const STEPS: &[StringMethodStep] = &[
    VirtualMachine::call_string_inspection_method,
    VirtualMachine::call_string_match_method,
    VirtualMachine::call_string_encode_method,
    VirtualMachine::call_string_case_method,
    VirtualMachine::call_string_trimming_method,
    VirtualMachine::call_string_byte_method,
    VirtualMachine::call_string_character_method,
    VirtualMachine::call_string_encoding_method,
    VirtualMachine::call_string_padding_method,
    VirtualMachine::call_string_slicing_method,
    VirtualMachine::call_string_searching_method,
    VirtualMachine::call_string_conversion_method,
    VirtualMachine::call_string_substitution_method,
];

impl VirtualMachine {
    /// The String method `method_name` names, run against `receiver`, or
    /// `None` when the receiver is not a String and no group answers.
    pub(crate) fn call_string_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let View::String(string_value) = receiver.view() else {
            return Ok(None);
        };
        self.with_subject_encoding(string_value, |vm| {
            for step in STEPS {
                if let Some(result) =
                    step(vm, receiver, string_value, method_name, arguments, position)?
                {
                    return Ok(Some(result));
                }
            }
            vm.call_string_set_method(receiver, method_name, arguments, position)
        })
    }

    /// `[]` or `slice`, named `method_name`, on the String `receiver`,
    /// which only the slicing group of String methods answers.
    pub(crate) fn call_string_slice(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        self.with_subject_encoding(string_value, |vm| {
            vm.call_string_slicing_method(receiver, string_value, method_name, arguments, position)
        })
    }

    /// Run `run` with `string_value`'s encoding as the one a match it
    /// records cuts its pieces out under.
    fn with_subject_encoding<T>(
        &mut self,
        string_value: &Rc<crate::object::StringValue>,
        run: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let held = self
            .match_subject_encoding
            .replace(string_value.encoding_name());
        let answered = run(self);
        self.match_subject_encoding = held;
        answered
    }
}
