// Putting the elements in order.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// Putting the elements in order.
    pub(crate) fn call_array_ordering_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "size" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(array_rc.borrow().len() as i64)))
            }
            "unshift" | "prepend" => {
                // Ruby: unshift(*items) prepends all items in order, accepts 0+ args
                let mut arr = array_rc.borrow_mut();
                for (i, item) in arguments.iter().enumerate() {
                    arr.insert(i, item.clone());
                }
                drop(arr);
                Ok(Some(receiver.clone()))
            }
            "sort" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // A block decides the order, answering negative, zero, or
                // positive the way `<=>` does.
                let viewed = self.pending_block.take();
                let block = match viewed.view() {
                    Some(View::Block(block)) => Some(block),
                    _ => None,
                };
                let elements = array_rc.borrow().clone();
                let sorted = self.sort_elements(elements, block.cloned(), position)?;
                Ok(Some(Object::array(sorted)))
            }
            "sort_by" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let viewed = self.pending_block.take();
                let block = match viewed.view() {
                    Some(View::Block(b)) => b,
                    // Without a block the sort is an Enumerator, which
                    // `sort_by.with_index` hands the index to.
                    None => {
                        return self
                            .make_enumerator(receiver, method_name, arguments, position)
                            .map(Some);
                    }
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other.to_object(),
                            position,
                        ));
                    }
                };
                let mut keyed: Vec<(Object, Object)> = Vec::new();
                for element in array_rc.borrow().iter() {
                    let key =
                        self.execute_block_callable(block, vec![element.clone()], position)?;
                    keyed.push((key, element.clone()));
                }
                let sorted = self.sort_by_keys(keyed, position)?;
                Ok(Some(Object::Array(Rc::new(RefCell::new(sorted)))))
            }
            "reverse" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut reversed = array_rc.borrow().clone();
                reversed.reverse();
                Ok(Some(Object::Array(Rc::new(RefCell::new(reversed)))))
            }
            _ => Ok(None),
        }
    }
}
