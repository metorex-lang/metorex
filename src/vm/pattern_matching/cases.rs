// Running a `case` statement, whether it matches on values or on
// patterns.

use super::*;
use crate::object::View;

impl VirtualMachine {
    /// Ruby names a `when` clause written twice with the same literal, since
    /// the second one can never be reached.
    pub(crate) fn warn_duplicated_when_clauses(
        &mut self,
        cases: &[crate::ast::MatchCase],
        position: Position,
    ) -> Result<(), MetorexError> {
        let mut seen: Vec<(String, usize)> = Vec::new();
        for case in cases {
            let Some(spelling) = literal_pattern_spelling(&case.pattern) else {
                continue;
            };
            let line = case.position.line;
            if let Some((_, first)) = seen.iter().find(|(held, _)| *held == spelling) {
                let message = format!(
                    "warning: 'when' clause on line {} duplicates 'when' clause on line {} and is ignored\n",
                    line, first
                );
                self.warn_through_warning_module(message, position)?;
                continue;
            }
            seen.push((spelling, line));
        }
        Ok(())
    }

    /// Evaluate a guard expression with pattern bindings in a new scope.
    /// Returns true if the guard passes (is truthy) or if there's no guard.
    /// Returns false if the guard fails (is not truthy).
    /// Helper method to reduce code duplication between match statement and case expression.
    pub(crate) fn evaluate_guard_with_bindings(
        &mut self,
        guard_expr: Option<&Expression>,
        bindings: &HashMap<String, Object>,
    ) -> Result<bool, MetorexError> {
        if let Some(guard) = guard_expr {
            // Push a new scope for guard evaluation with bindings
            self.environment_mut().push_scope();
            self.apply_pattern_bindings(bindings);

            let guard_result = self.evaluate_expression(guard);
            self.environment_mut().pop_scope();

            // Return whether the guard is truthy
            Ok(is_truthy(&guard_result?))
        } else {
            // No guard means it always passes
            Ok(true)
        }
    }

    /// Test one `when` arm of a `case`/`when` statement against `subject`.
    /// A match is counted for coverage and binds what the pattern bound in
    /// the scope the `case` is in.
    pub(crate) fn enter_when_arm(
        &mut self,
        arm: &crate::bytecode::unit::PatternArm,
        subject: &Object,
    ) -> Result<bool, MetorexError> {
        let mut bindings: HashMap<String, Object> = HashMap::new();
        if !self.match_pattern(&arm.pattern, subject, &mut bindings, arm.case_position)?
            || !self.evaluate_guard_with_bindings(arm.guard.as_ref(), &bindings)?
        {
            return Ok(false);
        }
        self.coverage_branch(arm.case_position, arm.index);
        self.apply_pattern_bindings(&bindings);
        Ok(true)
    }

    /// Test one `in` arm's pattern of a `case`/`in` statement against
    /// `subject`. The names the pattern reached are locals of the scope the
    /// `case` is in, whether or not it matched, the ones it did not bind
    /// holding nil.
    pub(crate) fn enter_in_arm(
        &mut self,
        arm: &crate::bytecode::unit::PatternArm,
        subject: &Object,
    ) -> Result<bool, MetorexError> {
        let mut bindings: HashMap<String, Object> = HashMap::new();
        self.pattern_failure = None;
        let matched =
            self.match_pattern(&arm.pattern, subject, &mut bindings, arm.case_position)?;
        self.apply_pattern_bindings(&bindings);
        Ok(matched)
    }

    /// Test one arm of a `case` expression against `subject`. When it
    /// matches, count it for coverage and open a scope holding what its
    /// pattern bound, in which its body runs.
    pub(crate) fn enter_case_arm(
        &mut self,
        arm: &crate::bytecode::unit::CaseArm,
        subject: &Object,
    ) -> Result<bool, MetorexError> {
        let mut bindings: HashMap<String, Object> = HashMap::new();
        if !self.match_pattern(&arm.pattern, subject, &mut bindings, arm.position)?
            || !self.evaluate_guard_with_bindings(arm.guard.as_ref(), &bindings)?
        {
            return Ok(false);
        }
        self.coverage_branch(arm.case_position, arm.index);
        self.environment_mut().push_scope();
        self.apply_pattern_bindings(&bindings);
        if let Some(line) = arm.counted_line
            && self.coverage.is_some()
        {
            self.coverage_count(line);
        }
        Ok(true)
    }
}

/// How a `when` pattern is written, for the literals that can be compared as
/// text. Anything else answers None, since two expressions that happen to
/// name the same value are not a duplicate as written.
pub(crate) fn literal_pattern_spelling(pattern: &crate::ast::MatchPattern) -> Option<String> {
    use crate::ast::{Expression, MatchPattern};
    match pattern {
        MatchPattern::IntLiteral(held) => Some(format!("i{held}")),
        MatchPattern::StringLiteral(held) => Some(format!("s{held}")),
        MatchPattern::SymbolLiteral(held) => Some(format!("y{held}")),
        MatchPattern::BoolLiteral(held) => Some(format!("b{held}")),
        MatchPattern::NilLiteral => Some("nil".to_string()),
        MatchPattern::Expression(held) => match held.as_ref() {
            Expression::IntLiteral { value, .. } => Some(format!("i{value}")),
            Expression::StringLiteral { value, .. } => Some(format!("s{value}")),
            Expression::Symbol { value, .. } => Some(format!("y{value}")),
            Expression::BoolLiteral { value, .. } => Some(format!("b{value}")),
            _ => None,
        },
        _ => None,
    }
}

/// Whether two names stand for the very same object, which is what tells the
/// value a `case` is matching from one nested inside a pattern.
pub(crate) fn names_the_same_value(left: &Object, right: &Object) -> bool {
    match (left.view(), right.view()) {
        (View::Array(one), View::Array(other)) => Rc::ptr_eq(one, other),
        (View::Dict(one), View::Dict(other)) => Rc::ptr_eq(one, other),
        (View::Instance(one), View::Instance(other)) => Rc::ptr_eq(one, other),
        (View::String(one), View::String(other)) => Rc::ptr_eq(one, other),
        _ => left == right,
    }
}
