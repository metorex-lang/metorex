// Reading and writing the constants a module holds.

use super::super::run_example;
#[test]
fn test_metaprogramming_const_added_hook_execution() {
    let expected = "[:TEST]\n[:TEST, :SECOND]\n[:TEST, :SECOND, :Autoload]\n[:TEST, :SECOND, :Autoload, :Child]\n[:TEST, :SECOND, :Autoload, :Child, :DIRECT]\n";
    let output = run_example("metaprogramming/const_added_hook.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_added_hook_parens_execution() {
    let expected = "[:TEST]\n[:TEST, :SECOND]\n[:TEST, :SECOND, :Autoload]\n[:TEST, :SECOND, :Autoload, :Child]\n[:TEST, :SECOND, :Autoload, :Child, :DIRECT]\n";
    let output = run_example("metaprogramming/const_added_hook_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_defined_execution() {
    let expected = "true\ntrue\ntrue\nfalse\nfalse\ntrue\ntrue\nfalse\ntrue\nfalse\nfalse\nNameError\nNameError\n";
    let output = run_example("metaprogramming/const_defined.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_defined_parens_execution() {
    let expected = "true\ntrue\ntrue\nfalse\nfalse\ntrue\ntrue\nfalse\ntrue\nfalse\nfalse\nNameError\nNameError\n";
    let output = run_example("metaprogramming/const_defined_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_get_execution() {
    let expected = ":from_parent\n:from_module\n:top\n:from_parent\n:top\n[:missing, :ANYTHING]\n:FROM_PARENT\nNameError\n";
    let output = run_example("metaprogramming/const_get.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_get_parens_execution() {
    let expected = ":from_parent\n:from_module\n:top\n:from_parent\n:top\n[:missing, :ANYTHING]\n:FROM_PARENT\nNameError\n";
    let output = run_example("metaprogramming/const_get_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_missing_execution() {
    let expected = "handled Anything\nhandled Direct\nuninitialized constant Bare::Nope\n:Nope\nuninitialized constant Bare::AlsoMissing\n";
    let output = run_example("metaprogramming/const_missing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_missing_parens_execution() {
    let expected = "handled Anything\nhandled Direct\nuninitialized constant Bare::Nope\n:Nope\nuninitialized constant Bare::AlsoMissing\n";
    let output = run_example("metaprogramming/const_missing_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_set_execution() {
    let expected =
        "nil\nNamedRoot\nNamedRoot::B\nNamedRoot::B::C\ntrue\n41\nNameError\nFrozenError\n";
    let output = run_example("metaprogramming/const_set.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_set_parens_execution() {
    let expected =
        "nil\nNamedRoot\nNamedRoot::B\nNamedRoot::B::C\ntrue\n41\nNameError\nFrozenError\n";
    let output = run_example("metaprogramming/const_set_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_source_location_execution() {
    let expected = "true\ntrue\ntrue\n[\"virtual.rb\", 100]\n[]\nnil\ntrue\nnil\n";
    let output = run_example("metaprogramming/const_source_location.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_source_location_parens_execution() {
    let expected = "true\ntrue\ntrue\n[\"virtual.rb\", 100]\n[]\nnil\ntrue\nnil\n";
    let output = run_example("metaprogramming/const_source_location_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_deprecate_constant_execution() {
    let expected = concat!(
        "true\n",
        "false\n",
        ":old\n",
        ":kept\n",
        ":old\n",
        "true\n",
        ":old\n",
        "NameError\n",
        "private\n",
        ":hidden\n"
    );
    let output = run_example("metaprogramming/deprecate_constant.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_deprecate_constant_no_parens_execution() {
    let expected = "2\n1\nprivate\n";
    let output = run_example("metaprogramming/deprecate_constant_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_added_top_level_execution() {
    let expected = concat!(
        "added TopLevelModule to Object\n",
        "added TopLevelClass to Object\n",
        "added TopLevelConstant to Object\n",
        "added Outer to Object\n",
        "added Inner to Outer\n",
        "added Nested to Outer\n",
        "added AnonymousBound to Object\n",
        "AnonymousBound\n",
    );
    let output = run_example("metaprogramming/const_added_top_level.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_added_top_level_parens_execution() {
    let expected = concat!(
        "added TopLevelModule to Object\n",
        "added TopLevelClass to Object\n",
        "added TopLevelConstant to Object\n",
        "added Outer to Object\n",
        "added Inner to Outer\n",
        "added Nested to Outer\n",
        "added AnonymousBound to Object\n",
        "AnonymousBound\n",
    );
    let output = run_example("metaprogramming/const_added_top_level_parens.rb");
    assert_eq!(output, expected);
}

/// The expected output of both `metaprogramming/constant_lookup` variants.
const CONSTANT_LOOKUP_OUTPUT: &str = "3\n3\n:from_base\n:missing_UNDEFINED_HERE\n\"uninitialized constant #<Settings module>::ABSENT\"\n:hidden\n\"constant\"\nnil\n[\"private constant Vault::SECRET referenced\", :SECRET, Vault]\n\"private constant Vault::Inner referenced\"\ntrue\n:top\n:top\n";

#[test]
fn test_metaprogramming_constant_lookup_execution() {
    let output = run_example("metaprogramming/constant_lookup.rb");
    assert_eq!(output, CONSTANT_LOOKUP_OUTPUT);
}

#[test]
fn test_metaprogramming_constant_lookup_no_parens_execution() {
    let output = run_example("metaprogramming/constant_lookup_no_parens.rb");
    assert_eq!(output, CONSTANT_LOOKUP_OUTPUT);
}

/// The expected output of both `metaprogramming/method_added_chain` variants.
const METHOD_ADDED_CHAIN_OUTPUT: &str = concat!(
    "[[:region=, :currency, :currency=, :load_rates, :store_rates], 5, :store_rates]\n",
    "[:method_added, :seen, :count, :last, :singletons, :refresh]\n",
);

#[test]
fn test_metaprogramming_method_added_chain_execution() {
    let output = run_example("metaprogramming/method_added_chain.rb");
    assert_eq!(output, METHOD_ADDED_CHAIN_OUTPUT);
}

#[test]
fn test_metaprogramming_method_added_chain_no_parens_execution() {
    let output = run_example("metaprogramming/method_added_chain_no_parens.rb");
    assert_eq!(output, METHOD_ADDED_CHAIN_OUTPUT);
}

/// The expected output of both `metaprogramming/extend_sees_later_methods` variants.
const EXTEND_SEES_LATER_METHODS_OUTPUT: &str = concat!(":second_version\n", ":second_version\n",);

#[test]
fn test_metaprogramming_extend_sees_later_methods_execution() {
    let output = run_example("metaprogramming/extend_sees_later_methods.rb");
    assert_eq!(output, EXTEND_SEES_LATER_METHODS_OUTPUT);
}

#[test]
fn test_metaprogramming_extend_sees_later_methods_no_parens_execution() {
    let output = run_example("metaprogramming/extend_sees_later_methods_no_parens.rb");
    assert_eq!(output, EXTEND_SEES_LATER_METHODS_OUTPUT);
}

#[test]
fn test_a_method_defined_from_a_block_reads_constants_where_the_block_was_written() {
    let expected = "[:feature, \"from Helper\", 1]\n[:feature, \"from Helper\", 2]\n\"found\"\n\"found\"\n\"found\"\n\"found\"\n\"found\"\n";
    let output = run_example("constants/define_singleton_method_block_cref.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_a_block_run_by_class_exec_reads_constants_where_it_was_written() {
    let expected = "\"top-level Diff::LCS\"\n\"top-level Diff::LCS\"\n\"top-level Diff::LCS\"\n";
    let output = run_example("constants/class_exec_block_lexical_lookup.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_a_when_constant_in_a_called_method_reads_where_the_method_was_written() {
    let output = run_example("constants/when_constant_in_called_method.rb");
    assert_eq!(output, ":array\n");
}

#[test]
fn test_module_reopens_the_module_a_top_level_alias_names() {
    let output = run_example("constants/reopen_module_through_alias.rb");
    assert_eq!(output, "[true, :added, :configured]\n[false, true]\n");
}

#[test]
fn test_include_and_extend_of_a_top_level_constant_skip_the_nested_one() {
    let output = run_example("constants/mixin_top_level_constant.rb");
    assert_eq!(output, "[:top_level, :top_level]\n");
}

#[test]
fn test_basic_object_class_exec_reads_constants_where_the_block_was_written() {
    let output = run_example("constants/basic_object_class_exec_reads_lexical_constants.rb");
    assert_eq!(output, "Class\n:outer_local\n");
}

#[test]
fn test_def_on_a_constant_in_a_class_body_opened_in_a_block_reaches_the_constant() {
    let output = run_example("constants/def_on_constant_in_block_class_body.rb");
    assert_eq!(
        output,
        "[:class_singleton, []]\n[:class_own, []]\n[:receiver_singleton, []]\n[:other_method]\n"
    );
}
