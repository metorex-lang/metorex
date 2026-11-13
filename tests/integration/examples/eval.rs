use super::run_example;

const SCOPE_AND_RETURNS: &str = "\"Cabinet::Shelf\"\nKernel\ntrue\n6\nLocalJumpError\n:from_the_eval\nLocalJumpError\nnil\n#<Encoding:BINARY (ASCII-8BIT)>\ntrue\n12\n42\n\"wrong argument type proc (expected binding)\"\n";

#[test]
fn test_eval_scope_and_returns_execution() {
    let output = run_example("eval/eval_scope_and_returns.rb");
    assert_eq!(output, SCOPE_AND_RETURNS);
}

#[test]
fn test_eval_scope_and_returns_no_parens_execution() {
    let output = run_example("eval/eval_scope_and_returns_no_parens.rb");
    assert_eq!(output, SCOPE_AND_RETURNS);
}

/// The expected output of both `eval/binding_paths` variants.
const BINDING_PATHS_OUTPUT: &str = "true\ntrue\ntrue\ntrue\n";

#[test]
fn test_eval_binding_paths_execution() {
    let output = run_example("eval/binding_paths.rb");
    assert_eq!(output, BINDING_PATHS_OUTPUT);
}

#[test]
fn test_eval_binding_paths_no_parens_execution() {
    let output = run_example("eval/binding_paths_no_parens.rb");
    assert_eq!(output, BINDING_PATHS_OUTPUT);
}

/// The expected output of both `eval/locals_read_while_parsing` variants.
const LOCALS_READ_WHILE_PARSING_OUTPUT: &str = concat!(
    "\"[:method, [[1]]]\"\n",
    "\"20\"\n",
    ":\"10\"\n",
    "/10/\n",
    "\"1\"\n",
    "20\n",
    "10\n",
    "20\n",
    "10\n",
    "4\n",
    "\"[:method, [[1]]]\"\n",
);

#[test]
fn test_eval_locals_read_while_parsing_execution() {
    let output = run_example("eval/locals_read_while_parsing.rb");
    assert_eq!(output, LOCALS_READ_WHILE_PARSING_OUTPUT);
}

#[test]
fn test_eval_locals_read_while_parsing_no_parens_execution() {
    let output = run_example("eval/locals_read_while_parsing_no_parens.rb");
    assert_eq!(output, LOCALS_READ_WHILE_PARSING_OUTPUT);
}

/// The expected output of both `eval/syntax_error_reports` variants.
const SYNTAX_ERROR_REPORTS: &str = concat!(
    "(eval):1: syntax error found\n",
    "> 1 | proc { _1 = 0 }\n",
    "    |        ^~ _1 is reserved for numbered parameters\n",
    "(eval):1: syntax errors found\n",
    "> 1 | x = 1 +\n",
    "    |        ^ unexpected end-of-input, assuming it is closing the parent top level context\n",
    "    |        ^ unexpected end-of-input; expected an expression after the operator\n",
    "(eval):4: syntax errors found\n",
    "  2 | second\n",
    "  3 | third\n",
    "> 4 | value = = 1\n",
    "    |       ^ expected an expression after `=`\n",
    "    |         ^ unexpected '='; target cannot be written\n",
    "    |           ^ unexpected integer, expecting end-of-input\n",
    "  5 | fourth\n",
    "  6 | fifth\n",
    "(eval):1: syntax errors found\n",
    "> 1 | ... + ) + 1\n",
    "    |     ^ unexpected ')', expected a receiver for unary `+`\n",
    "    |       ^ unexpected ')', ignoring it\n",
    "    |       ^ unexpected ')', expecting end-of-input\n",
    "    |         ^ unexpected '+', ignoring it\n",
    "(eval):1: syntax errors found\n",
    "> 1 | ä = 1 +\n",
    "    |        ^ unexpected end-of-input, assuming it is closing the parent top level context\n",
    "    |        ^ unexpected end-of-input; expected an expression after the operator\n",
    "named.rb:10: syntax errors found\n",
    "> 10 | 1 +\n",
    "     |    ^ unexpected end-of-input, assuming it is closing the parent top level context\n",
    "     |    ^ unexpected end-of-input; expected an expression after the operator\n",
);

#[test]
fn test_eval_syntax_error_reports_execution() {
    let output = run_example("eval/syntax_error_reports.rb");
    assert_eq!(output, SYNTAX_ERROR_REPORTS);
}

#[test]
fn test_eval_syntax_error_reports_no_parens_execution() {
    let output = run_example("eval/syntax_error_reports_no_parens.rb");
    assert_eq!(output, SYNTAX_ERROR_REPORTS);
}

/// The expected output of both `eval/binding_in_module_eval` variants.
const BINDING_IN_MODULE_EVAL: &str =
    concat!("[:answer]\n", "[:define_on]\n", "42\n", "\"2 items\\n\"\n",);

#[test]
fn test_eval_binding_in_module_eval_execution() {
    let output = run_example("eval/binding_in_module_eval.rb");
    assert_eq!(output, BINDING_IN_MODULE_EVAL);
}

#[test]
fn test_eval_binding_in_module_eval_no_parens_execution() {
    let output = run_example("eval/binding_in_module_eval_no_parens.rb");
    assert_eq!(output, BINDING_IN_MODULE_EVAL);
}

/// The expected output of both `eval/refused_programs` variants.
const REFUSED_PROGRAMS: &str = concat!(
    "(eval):1: syntax error found\n",
    "> 1 | 1 2\n",
    "    |   ^ unexpected integer, expecting end-of-input\n",
    "(eval):2: syntax error found\n",
    "  1 | first = 1\n",
    "> 2 | second = 2 3\n",
    "    |            ^ unexpected integer, expecting end-of-input\n",
    "(eval):1: syntax error found\n",
    "> 1 | def pair(left, left); end\n",
    "    |                ^~~~ duplicated argument name\n",
    "2\n",
);

#[test]
fn test_eval_refused_programs_execution() {
    let output = run_example("eval/refused_programs.rb");
    assert_eq!(output, REFUSED_PROGRAMS);
}

#[test]
fn test_eval_refused_programs_no_parens_execution() {
    let output = run_example("eval/refused_programs_no_parens.rb");
    assert_eq!(output, REFUSED_PROGRAMS);
}

#[test]
fn test_eval_locals_through_bindings_execution() {
    let output = run_example("eval/locals_through_bindings.rb");
    assert_eq!(
        output,
        "2\n3\n[:first, :second]\n5\n[:early, :late]\nfalse\n6\n8\nfalse\n"
    );
}

#[test]
fn eval_counts_lines_from_zero_when_told_to() {
    let output = run_example("eval/eval_from_line_zero.rb");
    assert_eq!(
        output,
        concat!(
            "(template):1:in 'Object#first_frame'\n(template):in 'Object#first_frame'\n",
            "(template):1:in 'Object#first_frame'\n(template):12:in 'Object#first_frame'\n"
        )
    );
}

#[test]
fn a_block_made_through_a_binding_reads_its_class_variables() {
    let output = run_example("eval/class_variable_through_a_binding.rb");
    assert_eq!(output, "2\n2\n");
}

#[test]
fn index_assignment_on_a_scoped_constant_calls_its_singleton_setter() {
    let output = run_example("eval/scoped_constant_index_assignment.rb");
    assert_eq!(output, "set local=0\nset inverted=1\nset top=2\n");
}

#[test]
fn an_exception_extended_with_a_module_is_one_of_that_module() {
    let output = run_example("eval/module_case_equality_on_extended_object.rb");
    assert_eq!(output, "true\ntrue\ntrue\ntrue\n\"Error\"\n");
}

#[test]
fn including_a_module_skips_the_modules_the_chain_already_holds() {
    let output = run_example("eval/include_of_a_module_already_in_the_ancestors.rb");
    assert_eq!(
        output,
        "[Holder, Later, Wrapper, Base, Object]\n\"wrapper then base\"\nWrapper\n"
    );
}

#[test]
fn a_call_with_arguments_reaches_the_method_a_proc_local_is_named_after() {
    let output = run_example("eval/method_call_named_like_a_proc_local.rb");
    assert_eq!(
        output,
        "\"blah1\"\nProc\n\"|blah1\"\n0\n#<MatchData \"blah\">\n"
    );
}

#[test]
fn a_subclass_is_a_kind_of_the_modules_its_superclass_was_extended_with() {
    let output = run_example("eval/kind_of_a_module_a_superclass_was_extended_with.rb");
    assert_eq!(
        output,
        "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n0\ntrue\ntrue\ntrue\ntrue\ntrue\n"
    );
}

#[test]
fn a_block_handed_on_to_class_eval_reads_constants_through_the_scopes_it_was_written_in() {
    let output = run_example("eval/class_eval_of_a_block_handed_on_keeps_its_nesting.rb");
    assert_eq!(
        output,
        "\"uninitialized constant Outer::NOPE\"\n\"uninitialized constant Outer::NOPE\"\n\"uninitialized constant Outer::NOPE\"\n\"uninitialized constant Outer::NOPE\"\n[Outer]\n[Outer]\n[Outer]\n"
    );
}

#[test]
fn toplevel_binding_reads_top_level_constants() {
    let output = run_example("eval/toplevel_binding_reads_top_level_constants.rb");
    assert_eq!(output, "\"1.5.0\"\n[]\n");
}

#[test]
fn begin_runs_first_in_every_eval() {
    let output = run_example("eval/begin_runs_first_in_every_eval.rb");
    assert_eq!(output, "true\ntrue\n[]\n");
}

#[test]
fn bindings_name_the_file_their_block_was_written_in() {
    let output = run_example("eval/bindings_in_loaded_blocks/main.rb");
    assert_eq!(
        output,
        "\"examples.rb\"\n\"examples.rb\"\n\"examples.rb:9)\"\n"
    );
}

#[test]
fn bindings_answer_their_methods() {
    let output = run_example("eval/bindings_answer_their_methods.rb");
    assert_eq!(output, "true\ntrue\ntrue\nBinding\n");
}

#[test]
fn bindings_inspect_as_objects() {
    let output = run_example("eval/bindings_inspect_as_objects.rb");
    assert_eq!(output, "true\ntrue\n");
}

#[test]
fn defs_through_bindings_install_where_the_binding_was_taken() {
    let output = run_example("eval/defs_through_bindings_install_where_the_binding_was_taken.rb");
    assert_eq!(output, "Object\n[]\n");
}

#[test]
fn class_eval_strings_keep_the_callers_scopes() {
    let output = run_example("eval/class_eval_strings_keep_the_callers_scopes.rb");
    assert_eq!(
        output,
        ":top_level\n[BasicObject, Object]\n\"uninitialized constant BasicObject::Store\"\n:top_level\n"
    );
}

#[test]
fn method_bindings_read_top_level_constants() {
    let output = run_example("eval/method_bindings_read_top_level_constants.rb");
    assert_eq!(output, "BasicObject\n");
}

#[test]
fn locals_holding_methods_are_locals() {
    let output = run_example("eval/locals_holding_methods_are_locals.rb");
    assert_eq!(
        output,
        "[[:ledger, :held_total, :unbound], 42, :to_s]\n[:held]\n\"local-variable\"\n\"method\"\n"
    );
}

#[test]
fn proc_bindings_stand_where_the_block_was_written() {
    let output = run_example("eval/proc_bindings_stand_where_the_block_was_written.rb");
    assert_eq!(
        output,
        "\"proc_bindings_stand_where_the_block_was_written.rb\"\n1\n"
    );
}

#[test]
fn classes_defined_through_bindings_are_not_locals() {
    let output = run_example("eval/classes_defined_through_bindings_are_not_locals.rb");
    assert_eq!(output, "false\nTally\ntrue\n");
}

#[test]
fn test_class_eval_counts_lines_from_below_one_and_reads_a_dashed_magic_comment() {
    let output = run_example("eval/class_eval_source_with_a_negative_line_and_a_magic_comment.rb");
    assert_eq!(
        output,
        concat!(
            "[1, true, 2, 2]\n",
            "\"template.erb:6\"\n",
            "false\n",
            "true\n",
        )
    );
}

#[test]
fn test_class_eval_source_encoding_is_its_literals_encoding() {
    let output = run_example("eval/class_eval_source_encoding_sets_its_literals_encoding.rb");
    assert_eq!(
        output,
        concat!(
            "[#<Encoding:Shift_JIS>, \"ふが\"]\n",
            "[#<Encoding:Shift_JIS>, \"ふが\"]\n",
            "#<Encoding:US-ASCII>\n",
            "[#<Encoding:Shift_JIS>, #<Encoding:EUC-JP>]\n",
        )
    );
}

#[test]
fn test_class_eval_strings_read_constants_through_the_receiver_then_the_caller() {
    let output = run_example("eval/class_eval_string_scope_reaches_a_block_inside_it.rb");
    assert_eq!(
        output,
        concat!(
            "[\"Bob\", [Person, Compiler]]\n",
            "\"Bob\"\n",
            "\"compiler\"\n",
        )
    );
}
