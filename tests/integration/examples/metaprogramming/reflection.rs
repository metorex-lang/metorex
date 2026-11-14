// What a program reports about the call that is running.

use super::super::run_example;
#[test]
fn test_metaprogramming_ast_inspection_execution() {
    let expected = "add\n2\n1\nBinaryOp\n+\n1\nBinaryOp\n*\n1\n1\n";
    let output = run_example("metaprogramming/ast/inspection.rb");
    assert_eq!(output, expected);
}

// 14.4 — Reflection and Introspection

#[test]
fn test_metaprogramming_reflection_execution() {
    let expected = r#"=== class ===
Dog
String
Integer
Array

=== instance_of? ===
true
false

=== is_a? ===
true
true

=== respond_to? ===
true
true
false

=== methods ===
[:fetch, :speak]

=== send ===
Woof!
Fetching ball

=== send with symbol ===
Woof!
Fetching stick

=== send on built-in ===
HELLO
5
"#;
    let output = run_example("metaprogramming/reflection/reflection.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_reflection_no_parens_execution() {
    let expected = r#"=== class ===
Dog
String
Integer
Array

=== instance_of? ===
true
false

=== is_a? ===
true
true

=== respond_to? ===
true
true
false

=== methods ===
[:fetch, :speak]

=== send ===
Woof!
Fetching ball

=== send with symbol ===
Woof!
Fetching stick

=== send on built-in ===
HELLO
5
"#;
    let output = run_example("metaprogramming/reflection/reflection_no_parens.rb");
    assert_eq!(output, expected);
}

// 14.5 — AST Manipulation

#[test]
fn test_metaprogramming_ast_manipulation_execution() {
    let expected = r#"=== eval ===
6
50
30

=== eval define method ===
14

=== eval define class ===
(3, 4)

=== parse ===
Array
1

=== code generation ===
7
7
30

=== runtime modification ===
8
15
"#;
    let output = run_example("metaprogramming/ast/manipulation.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_ast_manipulation_no_parens_execution() {
    let expected = r#"=== eval ===
6
50
30

=== eval define method ===
14

=== eval define class ===
(3, 4)

=== parse ===
Array
1

=== code generation ===
7
7
30

=== runtime modification ===
8
15
"#;
    let output = run_example("metaprogramming/ast/manipulation_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_caller_locations_lineno_execution() {
    let expected = "true\ntrue\n";
    let output = run_example("metaprogramming/caller_locations_lineno.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_caller_locations_slicing_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "nil\n",
        "[]\n",
        "Thread::Backtrace::Location\n",
        "true\n",
        "true\n",
        "true\n",
    );
    let output = run_example("metaprogramming/caller_locations_slicing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_caller_locations_slicing_parens_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "nil\n",
        "[]\n",
        "Thread::Backtrace::Location\n",
        "true\n",
        "true\n",
        "true\n",
    );
    let output = run_example("metaprogramming/caller_locations_slicing_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_caller_entries_execution() {
    let expected = concat!(
        "true\n", "true\n", "true\n", "true\n", "true\n", "true\n", "nil\n", "[]\n", "true\n",
        "true\n",
    );
    let output = run_example("metaprogramming/caller_entries.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_caller_entries_parens_execution() {
    let expected = concat!(
        "true\n", "true\n", "true\n", "true\n", "true\n", "true\n", "nil\n", "[]\n", "true\n",
        "true\n",
    );
    let output = run_example("metaprogramming/caller_entries_parens.rb");
    assert_eq!(output, expected);
}

/// The expected output of both `metaprogramming/kernel_method_objects` variants.
const KERNEL_METHOD_OBJECTS_OUTPUT: &str = concat!(
    "#<UnboundMethod: Kernel#singleton_class()>\n",
    "#<Class:Ledger>\n",
    "String\n",
    "Kernel\n",
    "#<Class:Ledger>\n",
);

#[test]
fn test_metaprogramming_kernel_method_objects_execution() {
    let output = run_example("metaprogramming/kernel_method_objects.rb");
    assert_eq!(output, KERNEL_METHOD_OBJECTS_OUTPUT);
}

#[test]
fn test_metaprogramming_kernel_method_objects_no_parens_execution() {
    let output = run_example("metaprogramming/kernel_method_objects_no_parens.rb");
    assert_eq!(output, KERNEL_METHOD_OBJECTS_OUTPUT);
}

/// The expected output of both `metaprogramming/core_method_aliases` variants.
const CORE_METHOD_ALIASES_OUTPUT: &str = "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\n:balance\nfalse\ntrue\n";

#[test]
fn test_metaprogramming_core_method_aliases_execution() {
    let output = run_example("metaprogramming/core_method_aliases.rb");
    assert_eq!(output, CORE_METHOD_ALIASES_OUTPUT);
}

#[test]
fn test_metaprogramming_core_method_aliases_no_parens_execution() {
    let output = run_example("metaprogramming/core_method_aliases_no_parens.rb");
    assert_eq!(output, CORE_METHOD_ALIASES_OUTPUT);
}

#[test]
fn defined_finds_the_public_copy_module_function_makes() {
    let output = run_example("metaprogramming/defined_module_function.rb");
    assert_eq!(output, "\"method\"\n\"method\"\nnil\nfalse\n");
}

#[test]
fn undef_method_on_a_core_class_hides_an_object_method() {
    let output =
        run_example("metaprogramming/undef_method_on_a_core_class_hides_an_object_method.rb");
    assert_eq!(
        output,
        concat!(
            ":object\n",
            "\"undefined method 'with_held' for an instance of Integer\"\n",
            "false\n",
        )
    );
}

#[test]
fn core_class_ancestry_reaches_object() {
    let output = run_example("metaprogramming/core_class_ancestry_reaches_object.rb");
    assert_eq!(
        output,
        concat!(
            "true\n",
            "true\n",
            "true\n",
            "true\n",
            "true\n",
            "Object\n",
            "[Object, Kernel, BasicObject]\n",
            "nil\n",
            "BasicObject\n",
            "true\n",
            "[Object, Kernel, BasicObject]\n",
        )
    );
}

#[test]
fn a_later_extend_overrides_an_earlier_one() {
    let output = run_example("metaprogramming/later_extend_overrides_an_earlier_one.rb");
    assert_eq!(
        output,
        "\".ba\"\n[#<Class:Speaker>, Reversing, Speaking]\n\".dc\"\n"
    );
}

#[test]
fn a_dup_of_an_extended_module_takes_an_override() {
    let output = run_example("metaprogramming/dup_of_an_extended_module_takes_an_override.rb");
    assert_eq!(
        output,
        "\"?ba\"\n\"ba\"\n\"!ba\"\n\"ab\"\n\"ab\"\n\"abcd\"\n"
    );
}

#[test]
fn object_ids_count_up_from_eight() {
    let output = run_example("metaprogramming/object_ids_count_up_from_eight.rb");
    assert_eq!(output, "Integer\n8\ntrue\ntrue\ntrue\n4\n20\n0\n3\n1\n");
}

#[test]
fn only_asking_for_an_id_takes_one() {
    let output = run_example("metaprogramming/only_asking_for_an_id_takes_one.rb");
    assert_eq!(
        output,
        "{\"holder.hash\" => 8, \"Struct.new(:field).new(1).hash\" => 8}\n"
    );
}

#[test]
fn module_body_makes_core_private_methods_public() {
    let output = run_example("metaprogramming/module_body_makes_core_private_methods_public.rb");
    assert_eq!(output, "true\ntrue\ntrue\nfalse\n");
}

#[test]
fn test_metaprogramming_method_missing_answers_before_metorex_functions_execution() {
    let output = run_example("metaprogramming/method_missing_answers_before_metorex_functions.rb");
    assert_eq!(output, "[\"parsed [1, 2]\", \"asserted [1, 2]\"]\nArray\n");
}

#[test]
fn test_metaprogramming_kernel_module_functions_execution() {
    let output = run_example("metaprogramming/kernel_module_functions.rb");
    assert_eq!(output, "true\ntrue\ntrue\ntrue\ntrue\nfalse\nfalse\n");
}

#[test]
fn class_eval_block_reads_class_instance_variables() {
    let output = run_example("metaprogramming/class_eval_block_reads_class_instance_variables.rb");
    assert_eq!(
        output,
        "nil\n\"loaded\"\n\"loaded\"\nnil\n\"loaded\"\n\"loaded\"\n"
    );
}
