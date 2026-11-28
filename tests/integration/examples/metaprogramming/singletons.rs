// The class one object alone has.

use super::super::run_example;
#[test]
fn test_metaprogramming_singleton_hooks_added_execution() {
    let expected = concat!(
        "object gained singleton_method_added\n",
        "object gained by_def\n",
        "object gained in_singleton_body\n",
        "object gained aliased\n",
        "object gained by_define_method\n",
        "object gained by_define_singleton_method\n",
        "Host gained singleton_method_added\n",
        "Host gained class_side\n",
        "1\n",
        "true\n"
    );
    let output = run_example("metaprogramming/singleton_hooks/added.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_singleton_hooks_added_parens_execution() {
    let expected = concat!(
        "object gained singleton_method_added\n",
        "object gained by_def\n",
        "object gained in_singleton_body\n",
        "object gained aliased\n",
        "object gained by_define_method\n",
        "object gained by_define_singleton_method\n",
        "Host gained singleton_method_added\n",
        "Host gained class_side\n",
        "1\n",
        "true\n"
    );
    let output = run_example("metaprogramming/singleton_hooks/added_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_singleton_hooks_removed_execution() {
    let expected = concat!(
        "class lost to_remove\n",
        "false\n",
        "object lost gone\n",
        "false\n",
        "NameError\n"
    );
    let output = run_example("metaprogramming/singleton_hooks/removed.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_singleton_hooks_removed_parens_execution() {
    let expected = concat!(
        "class lost to_remove\n",
        "false\n",
        "object lost gone\n",
        "false\n",
        "NameError\n"
    );
    let output = run_example("metaprogramming/singleton_hooks/removed_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_singleton_hooks_undefined_execution() {
    let expected = concat!(
        "true\n",
        "class undefined to_undefine\n",
        "false\n",
        "NoMethodError after undef\n",
        "NameError\n"
    );
    let output = run_example("metaprogramming/singleton_hooks/undefined.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_singleton_hooks_undefined_parens_execution() {
    let expected = concat!(
        "true\n",
        "class undefined to_undefine\n",
        "false\n",
        "NoMethodError after undef\n",
        "NameError\n"
    );
    let output = run_example("metaprogramming/singleton_hooks/undefined_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_super_from_a_singleton_method_reaches_modules_the_singleton_includes() {
    let expected = "11\n16\n\"hello from an extended object\"\n";
    let output = run_example("metaprogramming/singleton_super_into_included_module.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_macros_define_methods_on_a_singleton_class() {
    let expected = "true\nfalse\n[:matches?]\n[]\n";
    let output = run_example("metaprogramming/singleton_class_macros.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_a_bound_kernel_method_finds_methods_an_object_overrides_method_for() {
    let expected = "Method\n-2\n[\"rent\", 5]\n\"a method of its own\"\n:record\n";
    let output = run_example("metaprogramming/bound_kernel_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn def_self_and_class_shovel_self_replace_each_other() {
    let output = run_example("metaprogramming/redefine_singleton_method_both_ways.rb");
    assert_eq!(output, "from def self\n10\nfrom singleton class body\n");
}

#[test]
fn test_class_method_super_from_subclass_reaches_extended_module_execution() {
    let output =
        run_example("metaprogramming/class_method_super_from_subclass_reaches_extended_module.rb");
    assert_eq!(output, "[:module, [:topics]]\n[\"base\", \"me3\"]\n");
}

#[test]
fn undef_in_instance_eval_on_a_class_undefines_a_method_of_the_class() {
    let output = run_example("metaprogramming/undef_in_instance_eval_on_a_class.rb");
    assert_eq!(
        output,
        "false\nfalse\nfalse\nfalse\n\"an active account\"\n"
    );
}
