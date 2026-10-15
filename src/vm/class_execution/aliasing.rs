// Pointing one method name at another outside a class body.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// Execute `alias new_name old_name` outside a class/module body. Ruby
    /// treats this as an alias on Object so it is visible in all instances.
    pub(crate) fn execute_alias(
        &mut self,
        new_name: &str,
        old_name: &str,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // `alias $ERROR_INFO $!` gives one global a second name, which reads
        // and writes the same value the first one does.
        if let (Some(new_global), Some(old_global)) =
            (new_name.strip_prefix('$'), old_name.strip_prefix('$'))
        {
            // A name that is itself a second name, as `$>` is for
            // `$stdout`, stands for the global it names.
            let original = self
                .global_aliases
                .get(old_global)
                .cloned()
                .unwrap_or_else(|| old_global.to_string());
            self.global_aliases.insert(new_global.to_string(), original);
            return Ok(ControlFlow::Next);
        }
        if let Some(enclosing) = self.def_scope_stack.last().cloned() {
            self.install_alias(&enclosing, new_name, old_name, position)?;
            self.invoke_class_hook(&enclosing, "method_added", new_name, position)?;
            return Ok(ControlFlow::Next);
        }
        if let Some(View::Class(object_class)) = self.globals().get("Object").view()
            && !object_class.alias_method(new_name, old_name)
        {
            return Err(MetorexError::runtime_error(
                format!("undefined method '{}' for alias", old_name),
                position_to_location(position),
            ));
        }
        Ok(ControlFlow::Next)
    }

    /// Evaluate attribute-name expressions for `attr_reader`/`attr_writer`/
    /// `attr_accessor`. Symbols and strings resolve directly; other values
    /// are coerced via `#to_str`. Mirrors Ruby's `Module#attr_*` semantics.
    pub(crate) fn resolve_attribute_names(
        &mut self,
        attributes: &[Expression],
        position: Position,
    ) -> Result<Vec<String>, MetorexError> {
        let mut names = Vec::with_capacity(attributes.len());
        for value in self.attribute_values(attributes, position)? {
            names.push(self.coerce_method_name(&value, "attr_accessor", position)?);
        }
        Ok(names)
    }

    /// The names an `attr_*` call is given, with each splat spread into the
    /// names its list holds. Parentheses leave a splat inside a group.
    pub(crate) fn attribute_values(
        &mut self,
        attributes: &[Expression],
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let mut values = Vec::with_capacity(attributes.len());
        for attribute in attributes {
            let mut attribute = attribute;
            while let Expression::Grouped { expression, .. } = attribute {
                attribute = expression;
            }
            if let Expression::Splat { expression, .. } = attribute {
                let spread = self.evaluate_expression(expression)?;
                match self.splat_through_to_a(spread, position)? {
                    Ok(items) => values.extend(items),
                    Err(held) => values.push(held),
                }
                continue;
            }
            values.push(self.evaluate_expression(attribute)?);
        }
        Ok(values)
    }
}
