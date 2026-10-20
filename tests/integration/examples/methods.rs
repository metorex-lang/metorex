use super::run_example;

#[test]
fn test_methods_keyword_args_execution() {
    let expected = "Hello, Alice!\nHi, Bob!\nHey, Carol!\n";
    let output = run_example("methods/keyword_args/keyword_args.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_keyword_args_parens_execution() {
    let expected = "Hello, Alice!\nHi, Bob!\nHey, Carol!\n";
    let output = run_example("methods/keyword_args/keyword_args_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_keyword_args_class_execution() {
    let expected = "I'm Alice, age 0\nI'm Bob, age 30\n";
    let output = run_example("methods/keyword_args/class.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_keyword_args_class_parens_execution() {
    let expected = "I'm Alice, age 0\nI'm Bob, age 30\n";
    let output = run_example("methods/keyword_args/class_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_global_variables_execution() {
    let expected = "3\nhello\n";
    let output = run_example("runtime/global/variables.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_global_variables_parens_execution() {
    let expected = "3\nhello\n";
    let output = run_example("runtime/global/variables_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_default_params_method_execution() {
    let expected = "Hello, Alice\nHi, Bob\n";
    let output = run_example("methods/default_params/method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_default_params_method_parens_execution() {
    let expected = "Hello, Alice\nHi, Bob\n";
    let output = run_example("methods/default_params/method_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_yield_qmark_call_execution() {
    let expected = "true\n";
    let output = run_example("methods/yield_qmark_call.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_splat_empty_call_execution() {
    let expected = "pong\n";
    let output = run_example("methods/splat_empty_call.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_callee_and_method_execution() {
    let expected = "[:plain, :plain]\n[:aliased, :plain]\n[:in_block, :in_block]\n:defined\n:from_send\nnil\nnil\nnil\nnil\nsuper-sub\n";
    let output = run_example("methods/callee_and_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_anonymous_block_parameter_execution() {
    let expected = "plain\nlabeled: value\n42\n";
    let output = run_example("methods/anonymous_block_parameter.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_singleton_keyword_method_names() {
    let expected = "Integer\nopened\nnot really\n42\nshoveled log\ntrue\nFloat\ntrue\n";
    let output = run_example("methods/singleton_keyword_method_names.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_singleton_keyword_method_names_no_parens() {
    let expected = "Integer\nopened\nnot really\n42\nshoveled log\ntrue\nFloat\ntrue\n";
    let output = run_example("methods/singleton_keyword_method_names_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_optional_before_required_binding_execution() {
    let expected = concat!(
        "<only\n",
        "[both\n",
        "(-)\n",
        "=-)\n",
        "{=}\n",
        "[1, 2, 3, 9]\n",
        "[1, 8, 3, 9]\n",
        "[1, 7, 8, 9]\n"
    );
    let output = run_example("methods/optional_before_required/binding.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_optional_before_required_binding_no_parens_execution() {
    let expected = concat!(
        "<only\n",
        "[both\n",
        "(-)\n",
        "=-)\n",
        "{=}\n",
        "[1, 2, 3, 9]\n",
        "[1, 8, 3, 9]\n",
        "[1, 7, 8, 9]\n"
    );
    let output = run_example("methods/optional_before_required/binding_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_scope_locals_stay_local_execution() {
    let expected = "inner\nouter\nouter\nouter\nclass body\nouter\n6\n";
    let output = run_example("methods/scope/locals_stay_local.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_scope_locals_stay_local_parens_execution() {
    let expected = "inner\nouter\nouter\nouter\nclass body\nouter\n6\n";
    let output = run_example("methods/scope/locals_stay_local_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_ruby2_keywords_flag_execution() {
    let expected = concat!(
        "2\n",
        "2\n",
        "undefined method 'missing' for class 'Forwarder'\n",
        "warned and carried on\n",
    );
    let output = run_example("methods/ruby2_keywords_flag.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_ruby2_keywords_flag_parens_execution() {
    let expected = concat!(
        "2\n",
        "2\n",
        "undefined method 'missing' for class 'Forwarder'\n",
        "warned and carried on\n",
    );
    let output = run_example("methods/ruby2_keywords_flag_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_argument_forwarding_execution() {
    let expected = "[1, 2, 3]\n[1, 2, 3]\n[1, 2]\n:done\n5\n";
    let output = run_example("methods/argument_forwarding.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_argument_forwarding_parens_execution() {
    let expected = "[1, 2, 3]\n[1, 2, 3]\n[1, 2]\n:done\n5\n";
    let output = run_example("methods/argument_forwarding_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_endless_definitions_execution() {
    let expected = "42\n0\n42\n3\n43\nnamed\n";
    let output = run_example("methods/endless_definitions.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_endless_definitions_parens_execution() {
    let expected = "42\n0\n42\n3\n43\nnamed\n";
    let output = run_example("methods/endless_definitions_parens.rb");
    assert_eq!(output, expected);
}

/// The expected output of both `methods/argument_binding` variants, which
/// differ only in whether the calls are written with parentheses.
const ARGUMENT_BINDING_OUTPUT: &str = concat!(
    "[1, 9, [], 2]\n",
    "[1, 2, [], 3]\n",
    "[1, 2, [3], 4]\n",
    "[1, 2, [], 3, nil]\n",
    "[1, 2, [], 3, 4]\n",
    "[1, 2, [3], 4, 5]\n",
    "[1, 2, 3, 4]\n",
    "SyntaxError\n",
    "no implicit conversion of Object into Proc\n",
);

#[test]
fn test_methods_argument_binding_execution() {
    let output = run_example("methods/argument_binding.rb");
    assert_eq!(output, ARGUMENT_BINDING_OUTPUT);
}

#[test]
fn test_methods_argument_binding_no_parens_execution() {
    let output = run_example("methods/argument_binding_no_parens.rb");
    assert_eq!(output, ARGUMENT_BINDING_OUTPUT);
}

/// The expected output of both `methods/default_that_redefines` variants.
const DEFAULT_THAT_REDEFINES_OUTPUT: &str = "42\n1\n\"hello\"\n[true, true]\nfalse\n";

#[test]
fn test_methods_default_that_redefines_execution() {
    let output = run_example("methods/default_that_redefines.rb");
    assert_eq!(output, DEFAULT_THAT_REDEFINES_OUTPUT);
}

#[test]
fn test_methods_default_that_redefines_no_parens_execution() {
    let output = run_example("methods/default_that_redefines_no_parens.rb");
    assert_eq!(output, DEFAULT_THAT_REDEFINES_OUTPUT);
}

/// The expected output of both `methods/nested_definitions` variants.
const NESTED_DEFINITIONS_OUTPUT: &str = "true\ntrue\n:entry\ntrue\n:prepared\nfalse\n:configured\nfalse\n:labeled\nFrozenError\n\"wrong number of arguments (given 0, expected 1..2)\"\nnil\nSyntaxError\nfalse\n";

#[test]
fn test_methods_nested_definitions_execution() {
    let output = run_example("methods/nested_definitions.rb");
    assert_eq!(output, NESTED_DEFINITIONS_OUTPUT);
}

#[test]
fn test_methods_nested_definitions_no_parens_execution() {
    let output = run_example("methods/nested_definitions_no_parens.rb");
    assert_eq!(output, NESTED_DEFINITIONS_OUTPUT);
}

/// The expected output of both `methods/call_arguments` variants.
const CALL_ARGUMENTS_OUTPUT: &str = "\"can't convert Refuses to Array (Refuses#to_a gives Integer)\"\nDeclines\n{\"name\" => 1, size: 2}\n{a: 4, b: 2, c: 7}\n[1, {\"a\" => 1, b: 2}]\n{a: 1}\n\"no keywords accepted\"\n[nil]\n[1, 3]\nSyntaxError\n[1, 2]\n";

#[test]
fn test_methods_call_arguments_execution() {
    let output = run_example("methods/call_arguments.rb");
    assert_eq!(output, CALL_ARGUMENTS_OUTPUT);
}

#[test]
fn test_methods_call_arguments_no_parens_execution() {
    let output = run_example("methods/call_arguments_no_parens.rb");
    assert_eq!(output, CALL_ARGUMENTS_OUTPUT);
}

/// The expected output of both `methods/super_forms` variants.
const SUPER_FORMS_OUTPUT: &str = "[[10, 2, :added, 5], {flag: :off}]\n[[10, 3, 4, :added, 5], {flag: :on}]\n[:changed, :b]\n[[], {}]\n[3, 4]\n\"implicit argument passing of super from method defined by define_method() is not supported. Specify all arguments explicitly.\"\n[:missing, :frozen?]\n[:relabeled, :named]\n[:wrapped, :plain]\n42\n";

#[test]
fn test_methods_super_forms_execution() {
    let output = run_example("methods/super_forms.rb");
    assert_eq!(output, SUPER_FORMS_OUTPUT);
}

#[test]
fn test_methods_super_forms_no_parens_execution() {
    let output = run_example("methods/super_forms_no_parens.rb");
    assert_eq!(output, SUPER_FORMS_OUTPUT);
}

const BARE_NAMES_OVER_KERNEL: &str = concat!(
    "report \n",
    "\"report \"\n",
    "\"wrong number of arguments (given 0, expected 1)\"\n",
);

#[test]
fn test_methods_bare_names_over_kernel_execution() {
    let output = run_example("methods/bare_names_over_kernel.rb");
    assert_eq!(output, BARE_NAMES_OVER_KERNEL);
}

#[test]
fn test_methods_bare_names_over_kernel_no_parens_execution() {
    let output = run_example("methods/bare_names_over_kernel_no_parens.rb");
    assert_eq!(output, BARE_NAMES_OVER_KERNEL);
}

const PRIVATE_TOP_LEVEL_METHODS: &str = concat!(
    "\"private method 'helper' called for an instance of Object\"\n",
    "\"private method 'helper' called for an instance of String\"\n",
    "\"private method 'helper' called for an instance of Integer\"\n",
    "\"private method 'helper' called for an instance of Array\"\n",
    "\"private method 'helper' called for nil\"\n",
    "\"private method 'helper' called for true\"\n",
    "\"private method 'helper' called for an instance of Symbol\"\n",
    "\"private method 'helper' called for an instance of Float\"\n",
    "\"private method 'helper' called for an instance of Hash\"\n",
    ":top\n",
    "[false, true]\n",
    "NoMethodError\n",
    ":top_level_mkdir\n",
    "0\n",
    "0\n",
    "true\n",
);

#[test]
fn test_methods_private_top_level_methods_execution() {
    let output = run_example("methods/private_top_level_methods.rb");
    assert_eq!(output, PRIVATE_TOP_LEVEL_METHODS);
}

#[test]
fn test_methods_private_top_level_methods_no_parens_execution() {
    let output = run_example("methods/private_top_level_methods_no_parens.rb");
    assert_eq!(output, PRIVATE_TOP_LEVEL_METHODS);
}

const YIELD_IN_METHOD_CLAUSES: &str = concat!(
    "\"recovered from first try\"\n",
    "ran in ensure\n",
    ":body\n",
    "\"answered from else\"\n",
);

#[test]
fn test_methods_yield_in_method_clauses_execution() {
    let output = run_example("methods/yield_in_method_clauses.rb");
    assert_eq!(output, YIELD_IN_METHOD_CLAUSES);
}

#[test]
fn test_methods_yield_in_method_clauses_no_parens_execution() {
    let output = run_example("methods/yield_in_method_clauses_no_parens.rb");
    assert_eq!(output, YIELD_IN_METHOD_CLAUSES);
}

const KEYWORD_METHOD_NAMES_OUTPUT: &str = concat!(
    "[\"in\", \"then\", \"do\", \"redo\", \"retry\"]\n",
    "[\"and\", \"or\", \"not\", \"unless\"]\n",
    "true\n",
    "false\n",
    "false\n",
);

#[test]
fn test_methods_keyword_method_names_execution() {
    let output = run_example("methods/keyword_method_names.rb");
    assert_eq!(output, KEYWORD_METHOD_NAMES_OUTPUT);
}

#[test]
fn test_methods_keyword_method_names_no_parens_execution() {
    let output = run_example("methods/keyword_method_names_no_parens.rb");
    assert_eq!(output, KEYWORD_METHOD_NAMES_OUTPUT);
}

const ASSIGNMENT_AND_CONDITIONS_OUTPUT: &str = concat!(
    "{hits: 0}\n",
    "10\n",
    "\"ADA\"\n",
    "\"unknown\"\n",
    ":ok\n",
    "[42, true]\n",
    "[:small, :large]\n",
    ":second\n",
);

#[test]
fn test_methods_assignment_and_conditions_execution() {
    let output = run_example("methods/assignment_and_conditions.rb");
    assert_eq!(output, ASSIGNMENT_AND_CONDITIONS_OUTPUT);
}

#[test]
fn test_methods_assignment_and_conditions_no_parens_execution() {
    let output = run_example("methods/assignment_and_conditions_no_parens.rb");
    assert_eq!(output, ASSIGNMENT_AND_CONDITIONS_OUTPUT);
}

/// The expected output of both `methods/top_level_self_and_calls` variants.
const TOP_LEVEL_SELF_AND_CALLS_OUTPUT: &str = concat!(
    "main\n",
    "main\n",
    "main\n",
    "main\n",
    "Caller\n",
    "\"local\"\n",
    ":method\n",
    "undefined method 'count' for main\n",
    "\"007\"\n",
    "5\n",
);

#[test]
fn test_methods_top_level_self_and_calls_execution() {
    let output = run_example("methods/top_level_self_and_calls.rb");
    assert_eq!(output, TOP_LEVEL_SELF_AND_CALLS_OUTPUT);
}

#[test]
fn test_methods_top_level_self_and_calls_no_parens_execution() {
    let output = run_example("methods/top_level_self_and_calls_no_parens.rb");
    assert_eq!(output, TOP_LEVEL_SELF_AND_CALLS_OUTPUT);
}

/// The expected output of both `methods/main_private_methods` variants.
const MAIN_PRIVATE_METHODS: &str = concat!(
    "[:define_method, :include, :private, :public, :ruby2_keywords, :using]\n",
    "[:inspect, :to_s]\n",
    "[:define_method, :include, :private, :public, :ruby2_keywords, :using]\n",
    "[false, true]\n",
    ":hello\n",
    "false\n",
    "true\n",
    "[true, :loud]\n",
    "\"QUIET\"\n",
    "\"private method 'define_method' called for main\"\n",
);

#[test]
fn test_methods_main_private_methods_execution() {
    let output = run_example("methods/main_private_methods.rb");
    assert_eq!(output, MAIN_PRIVATE_METHODS);
}

#[test]
fn test_methods_main_private_methods_no_parens_execution() {
    let output = run_example("methods/main_private_methods_no_parens.rb");
    assert_eq!(output, MAIN_PRIVATE_METHODS);
}

/// The expected output of both `methods/alias_of_a_setter` variants.
const ALIAS_OF_A_SETTER: &str = concat!("\"02:00\"\n", "\"03:00\"\n", ":set_window\n",);

#[test]
fn test_methods_alias_of_a_setter_execution() {
    let output = run_example("methods/alias_of_a_setter.rb");
    assert_eq!(output, ALIAS_OF_A_SETTER);
}

#[test]
fn test_methods_alias_of_a_setter_no_parens_execution() {
    let output = run_example("methods/alias_of_a_setter_no_parens.rb");
    assert_eq!(output, ALIAS_OF_A_SETTER);
}

/// The expected output of both `methods/forms_gems_write` variants.
const FORMS_GEMS_WRITE: &str = concat!(
    "[\"sku\", \"sku: not found\"]\n",
    "\"4kg to north\"\n",
    "[OvernightShipment, ExpressShipment, 2]\n",
    "true\n",
    "[[1, 2], {}, :signed]\n",
    "[[3], {late: true}, nil]\n",
    "[5, 9]\n",
);

#[test]
fn test_methods_forms_gems_write_execution() {
    let output = run_example("methods/forms_gems_write.rb");
    assert_eq!(output, FORMS_GEMS_WRITE);
}

#[test]
fn test_methods_forms_gems_write_no_parens_execution() {
    let output = run_example("methods/forms_gems_write_no_parens.rb");
    assert_eq!(output, FORMS_GEMS_WRITE);
}

/// The expected output of both `methods/warn_parameters` variants.
const WARN_PARAMETERS_OUTPUT: &str = "[[:rest, :msgs], [:key, :uplevel], [:key, :category]]\n\"expected true or false as exception: nil\"\n\"expected true or false as exception: 0\"\n\"expected true or false as close_others: 1\"\n\"expected true or false as nochdir: \\\"true\\\"\"\n\"expected true or false as noclose: \\\"x\\\"\"\n";

#[test]
fn test_methods_warn_parameters_execution() {
    let output = run_example("methods/warn_parameters.rb");
    assert_eq!(output, WARN_PARAMETERS_OUTPUT);
}

#[test]
fn test_methods_warn_parameters_no_parens_execution() {
    let output = run_example("methods/warn_parameters_no_parens.rb");
    assert_eq!(output, WARN_PARAMETERS_OUTPUT);
}

#[test]
fn test_alias_accepts_keyword_method_names() {
    let expected = "ready and paid\nready or paid\nready unless paid\nready and paid\n";
    let output = run_example("methods/alias_keyword_names.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_bare_names_reach_method_missing() {
    let expected =
        "predicate valid\npredicate paid\npredicate within [3]\npredicate empty\nNameError\n";
    let output = run_example("methods/method_missing_for_bare_names.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_method_missing_frames_name_the_file_method_missing_is_written_in() {
    let expected = "method_missing_bare_runner.rb:5:in 'BarePredicates#method_missing'\n\
                    method_missing_bare_backtrace.rb:6:in 'Checker#check'\n\
                    method_missing_bare_backtrace.rb:10:in '<main>'\n\
                    method_missing_bare_runner.rb:5:in 'BarePredicates#method_missing'\n\
                    method_missing_bare_backtrace.rb:18:in 'ExplicitChecker#check'\n";
    let output = run_example("methods/method_missing_bare_backtrace.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_method_missing_super_words_the_receiver_as_ruby_does() {
    let expected = "undefined method 'shipped' for an instance of Forwarder\n\
                    undefined method 'shipped' for #<Forwarder:0x>\n";
    let output = run_example("methods/method_missing_super_wording.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_native_methods_report_the_signatures_mri_gives_them() {
    let expected = "[[:rest]]\n-1\n[[:rest]]\n-1\n[[:req]]\n1\n-1\n0\n2\n";
    let output = run_example("methods/native_method_signatures.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_a_method_object_a_statement_answers_is_not_called() {
    let expected = ":done\n:done\n:done\nhello\na Method held in a local is not called\n";
    let output = run_example("methods/discarded_method_object.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_a_bound_kernel_method_inside_a_rescue_finds_the_method() {
    let expected = "-2\n-2\nnil\n";
    let output = run_example("methods/bound_call_inside_rescue.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_a_default_value_reaches_the_block_the_method_was_called_with() {
    let expected = "[3, 5, 7]\n[1, 2, 3]\n:from_the_block\n:passed\ntrue\nfalse\n";
    let output = run_example("methods/yield_in_default_value.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_a_method_called_on_include_s_answer_reads_as_an_expression() {
    let output = run_example("methods/include_call_chain.rb");
    assert_eq!(output, "[:included, 2, 2]\n");
}

#[test]
fn test_a_trap_runs_for_a_signal_the_process_sends_itself() {
    let output = run_example("methods/trap_own_signal.rb");
    assert_eq!(output, "[:interrupted]\n");
}

#[test]
fn test_io_select_waits_for_a_pipe_another_process_writes() {
    let output = run_example("io/select_waits_for_another_process.rb");
    assert_eq!(output, "true\n[\"worker 1\", \"worker 0\"]\n");
}

#[test]
fn test_io_read_with_a_length_waits_for_every_byte_a_pipe_will_send() {
    let output = run_example("io/read_length_waits_for_every_byte.rb");
    assert_eq!(output, "[100000, 30000, 40000, nil]\n");
}

#[test]
fn test_undef_method_removes_every_method_a_class_inherits_from_object() {
    let output = run_example("methods/undef_inherited_core_methods.rb");
    assert_eq!(
        output,
        "[:__id__, :__send__, :class, :object_id, :respond_to?]\n"
    );
}

#[test]
fn test_a_module_answers_respond_to_without_its_own_instance_respond_to_missing() {
    let output = run_example("methods/module_respond_to_missing_is_for_instances.rb");
    assert_eq!(output, "false\nfalse\nfalse\ntrue\ntrue\nfalse\n");
}

#[test]
fn test_an_assignment_to_a_class_or_module_attribute_needs_a_class_level_writer() {
    let output = run_example("methods/setter_on_a_class_needs_a_class_method.rb");
    assert_eq!(
        output,
        "undefined method 'level=' for module Settings\n\
         undefined method 'size=' for class Widget\n\
         undefined method 'color=' for class Widget\n\
         []\n"
    );
}

#[test]
fn test_stderr_accepts_an_object_that_answers_write_through_method_missing() {
    let output = run_example("io/stderr_wrapper_answers_write_through_method_missing.rb");
    assert_eq!(
        output,
        "written through the wrapper\n$stderr must have write method, #<Class:0x> given\n"
    );
}

#[test]
fn test_a_class_hands_out_kernel_functions_as_methods() {
    let output = run_example("methods/kernel_function_method_on_a_class.rb");
    assert_eq!(
        output,
        "[:puts, Kernel]\ncalled through String\nKernel\nKernel\n:puts\nNameError\n"
    );
}

#[test]
fn test_a_call_of_an_undefined_name_reaches_method_missing() {
    let output = run_example("methods/undefined_names_reach_method_missing.rb");
    assert_eq!(
        output,
        "[:missing, :<=>, 1]\n[:missing, :==, 2]\n[:missing, :<=>, 3]\n[:missing, :<, 4]\n\
         \"custom match\"\nNoMethodError\n[:missing, :inspect]\n\
         \"undefined method 'hash' for an instance of Plain\"\n[:missing, :<, 5]\n"
    );
}

#[test]
fn test_a_binary_ampersand_at_the_end_of_a_line_continues_the_expression() {
    let output = run_example("parser/trailing_binary_ampersand.rb");
    assert_eq!(output, "2\n[2]\n7\n2\n");
}

#[test]
fn test_an_attribute_list_ends_at_the_newline_before_an_if() {
    let output = run_example("parser/attr_list_ends_at_newline.rb");
    assert_eq!(output, "[:backtrace, :cause, :cause=]\n");
}

#[test]
fn test_assignments_after_return_and_while_and_operands_after_end_and_lambda() {
    let output = run_example("parser/assignment_and_operand_positions.rb");
    assert_eq!(
        output,
        ":returned\n2\n[Proc, Proc]\n[:from_if, :from_or]\n:none\n"
    );
}

#[test]
fn test_class_new_taken_as_a_method_object_constructs_past_an_overriding_new() {
    let output = run_example("methods/class_new_as_a_method_object.rb");
    assert_eq!(output, "Guard\n0\nThread::Mutex\nPlain\n");
}

#[test]
fn test_a_rescue_naming_a_class_constant_asks_the_module_it_holds() {
    let output = run_example("rescue/module_case_equality_through_class_constant.rb");
    assert_eq!(output, "Catching\n[:caught, \"deep\"]\n");
}

#[test]
fn test_super_in_a_define_method_body_starts_past_the_class_it_was_defined_on() {
    let output = run_example("methods/define_method_super_from_a_subclass.rb");
    assert_eq!(
        output,
        "[:group, :from_module]\n[:group, :from_module]\nGroup\n"
    );
}

#[test]
fn test_super_in_a_block_starts_past_the_class_its_method_was_defined_on() {
    let output = run_example("methods/super_in_a_block_of_a_class_named_later.rb");
    assert_eq!(output, ":from_module\n");
}

#[test]
fn test_include_inside_an_instance_method_calls_the_object_s_own_include() {
    let output = run_example("methods/include_inside_an_instance_method.rb");
    assert_eq!(
        output,
        "[[Comparable, []], [Enumerable, [:tagged]]]\nfalse\n"
    );
}

#[test]
fn test_array_difference_matches_structs_by_eql() {
    let output = run_example("array_methods/difference_of_structs.rb");
    assert_eq!(output, "true\ntrue\ntrue\n0\n1\ntrue\n");
}

#[test]
fn test_object_eql_is_identity_and_array_difference_uses_it() {
    let output = run_example("methods/eql_and_array_difference.rb");
    assert_eq!(
        output,
        "true\nfalse\n[1, 3]\ntrue\n[#<struct Point x=3, y=4>]\ntrue\n"
    );
}

#[test]
fn test_a_def_in_a_block_instance_exec_runs_from_a_block_lands_on_the_receiver() {
    let output = run_example("methods/def_in_instance_exec_from_a_block.rb");
    assert_eq!(
        output,
        "[[:initialize], [:total], [], Group]\n[:initialize]\n"
    );
}

#[test]
fn test_a_keyword_value_may_stand_on_the_line_after_its_label() {
    let output = run_example("methods/keyword_args/value_on_the_next_line.rb");
    assert_eq!(
        output,
        "[:level, [:fatal, :warn], :info]\n[:mode, :plain, 1]\n[:short, :held, 2]\n{level: 2}\n{level: 1}\n"
    );
}

#[test]
fn test_an_evaluated_def_in_a_top_level_instance_exec_lands_on_the_receiver() {
    let output = run_example("methods/instance_exec_definee/top_level.rb");
    assert_eq!(output, "[:balance]\nfalse\n[:balance]\ntrue\n");
}

#[test]
fn test_parse_answers_a_node_for_each_kind_of_statement_and_expression() {
    let output = run_example("parser/node_shapes.rb");
    assert_eq!(
        output,
        concat!(
            "{\"type\" => \"InterpolatedString\"}\n",
            "{\"type\" => \"Unknown\"}\n",
            "{\"type\" => \"While\", \"condition\" => {\"type\" => \"Identifier\", \"name\" => \"x\"}, \"body\" => [{\"type\" => \"Identifier\", \"name\" => \"y\"}]}\n",
            "{\"type\" => \"For\", \"variable\" => \"i\", \"iterable\" => {\"type\" => \"Identifier\", \"name\" => \"list\"}, \"body\" => [{\"type\" => \"Identifier\", \"name\" => \"i\"}]}\n",
            "{\"type\" => \"Raise\", \"exception\" => {\"type\" => \"Identifier\", \"name\" => \"Error\"}}\n",
            "[{\"type\" => \"Continue\"}, {\"type\" => \"Break\"}]\n",
            "{\"type\" => \"Index\", \"array\" => {\"type\" => \"Identifier\", \"name\" => \"list\"}, \"index\" => {\"type\" => \"IntLiteral\", \"value\" => 0}}\n",
            "{\"type\" => \"UnaryOp\", \"op\" => \"-\", \"operand\" => {\"type\" => \"Identifier\", \"name\" => \"x\"}}\n",
            "{\"type\" => \"UnaryOp\", \"op\" => \"!\", \"operand\" => {\"type\" => \"Identifier\", \"name\" => \"x\"}}\n"
        )
    );
}

#[test]
fn test_kernel_methods_answer_bare_at_the_top_level() {
    let output = run_example("methods/bare_kernel_methods.rb");
    assert_eq!(
        output,
        "main\nfalse\ntrue\ntrue\n[]\ntrue\nmain1[1, :a]nil\n\"text\"\n"
    );
}

#[test]
fn an_equality_method_reaching_basic_object_through_super_compares_identity() {
    let expected = "true\ntrue\nfalse\ntrue\ntrue\n";
    let output = run_example("methods/equality_super_to_basic_object.rb");
    assert_eq!(output, expected);
}

#[test]
fn a_class_method_reaches_a_module_extended_by_a_call_through_super() {
    let output = run_example("methods/super_from_singleton_to_extended_module.rb");
    assert_eq!(output, "SHIPPING PALLET\nSHIPPING CRATE\n");
}

#[test]
fn ruby2_keywords_pass_through_new_and_send_with_a_block() {
    let output = run_example("methods/ruby2_keywords_through_new.rb");
    assert_eq!(output, "true\nfalse\nfalse\n");
}

#[test]
fn an_implicit_super_in_a_yielded_block_passes_its_own_methods_arguments() {
    let output = run_example("methods/zsuper_in_block_yielded_by_another_method.rb");
    assert_eq!(output, "fragile: crate\nrush: 3 fragile: crate\n");
}

#[test]
fn an_undefined_equality_operator_reaches_method_missing() {
    let output = run_example("methods/undef_equality_reaches_method_missing.rb");
    assert_eq!(output, "true\ntrue\nfalse\nfalse\ntrue\n");
}

#[test]
fn a_method_missing_mixed_into_object_leaves_kernel_methods_to_kernel() {
    let output = run_example("methods/native_methods_before_method_missing_on_object.rb");
    assert_eq!(
        output,
        "true\ntrue\ntrue\n\"fallback be_shipped\"\nundefined method 'unknown_label' for an instance of Object\n"
    );
}

#[test]
fn a_module_prepended_to_a_singleton_class_wraps_the_class_method() {
    let output = run_example("methods/prepend_to_singleton_class_wraps_class_method.rb");
    assert_eq!(
        output,
        "labeled shipment of 12\n[Labeled, #<Class:Shipment>]\n"
    );
}

#[test]
fn respond_to_missing_reaches_kernels_through_super_for_every_kind_of_object() {
    let output = run_example("methods/respond_to_missing_super_reaches_kernel.rb");
    assert_eq!(output, "true\nfalse\nfalse\nfalse\ntrue\n");
}
