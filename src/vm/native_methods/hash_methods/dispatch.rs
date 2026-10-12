// The order a Hash method is looked for in. Each step answers `None` when
// the name is none of its own, and the next step is tried.

use super::*;

/// One step of the chain: every group of Hash methods reads the same
/// arguments and answers the same way.
type HashMethodStep = fn(
    &mut VirtualMachine,
    &Object,
    &Rc<RefCell<IndexMap<String, Object>>>,
    &str,
    &[Object],
    Position,
) -> Result<Option<Object>, MetorexError>;

const STEPS: &[HashMethodStep] = &[
    VirtualMachine::call_hash_identity_method,
    VirtualMachine::call_hash_lookup_method,
    VirtualMachine::call_hash_default_method,
    VirtualMachine::call_hash_digging_method,
    VirtualMachine::call_hash_iteration_method,
    VirtualMachine::call_hash_comparing_method,
    VirtualMachine::call_hash_rebuilding_method,
    VirtualMachine::call_hash_transforming_method,
    VirtualMachine::call_hash_writing_method,
    VirtualMachine::call_hash_query_method,
];

impl VirtualMachine {
    /// The Hash method `method_name` names, run against `receiver`, or
    /// `None` when the receiver is not a Hash and no group answers.
    pub(crate) fn call_hash_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Dict(dict_rc) = receiver else {
            return Ok(None);
        };
        // ENV is a hash the process shares with the operating system, and a
        // handful of methods answer differently on it.
        if self.dict_is_environment(dict_rc)
            && let Some(result) = self.call_environment_hash_method(
                receiver,
                dict_rc,
                method_name,
                arguments,
                position,
            )?
        {
            return Ok(Some(result));
        }
        // A name a method takes out of ENV leaves the process environment
        // too, so a child process started later does not see it.
        let named_before: Option<Vec<String>> = self
            .dict_is_environment(dict_rc)
            .then(|| dict_rc.borrow().keys().cloned().collect());
        for step in STEPS {
            if let Some(result) = step(self, receiver, dict_rc, method_name, arguments, position)? {
                if let Some(named_before) = named_before {
                    let removed: Vec<String> = named_before
                        .into_iter()
                        .filter(|name| !dict_rc.borrow().contains_key(name))
                        .collect();
                    for name in removed {
                        let key = crate::vm::utils::dict_key_to_object(&name);
                        self.record_environment_change(dict_rc, &key, &Object::Nil);
                    }
                }
                return Ok(Some(result));
            }
        }
        Ok(None)
    }
}
