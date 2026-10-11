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
