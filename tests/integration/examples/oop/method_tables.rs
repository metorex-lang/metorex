// Adding, removing and renaming the methods of a class.

use super::super::run_example;
#[test]
fn test_oop_alias_method_strings_execution() {
    let output = run_example("oop/alias_method_strings.rb");
    assert_eq!(output, "original\noriginal\n");
}

#[test]
fn test_oop_unbound_method_execution() {
    let expected = ":from_module\n:from_base\n:from_base\n1\n:missing\n42 is not a symbol nor a string\n:label\n";
    let output = run_example("oop/unbound_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_unbound_method_parens_execution() {
    let expected = ":from_module\n:from_base\n:from_base\n1\n:missing\n42 is not a symbol nor a string\n:label\n";
    let output = run_example("oop/unbound_method_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_instance_methods_visibility_execution() {
    let expected = "[:protected_parent, :public_parent]\n[:public_parent]\n[:protected_parent]\n[:private_parent]\n[:public_child]\ntrue\nfalse\nNoMethodError\n";
    let output = run_example("oop/instance_methods_visibility.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_instance_methods_visibility_parens_execution() {
    let expected = "[:protected_parent, :public_parent]\n[:public_parent]\n[:protected_parent]\n[:private_parent]\n[:public_child]\ntrue\nfalse\nNoMethodError\n";
    let output = run_example("oop/instance_methods_visibility_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_method_added_hook_execution() {
    let expected = "[[:singleton, :singleton_method_added], [:added, :first], [:added, :aliased], [:added, :aliased_again], [:added, :inherited_method], [:added, :retired]]\nfalse\n[:aliased, :aliased_again, :first, :inherited_method]\nnil\ntrue\n";
    let output = run_example("oop/method_added_hook.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_method_added_hook_parens_execution() {
    let expected = "[[:singleton, :singleton_method_added], [:added, :first], [:added, :aliased], [:added, :aliased_again], [:added, :inherited_method], [:added, :retired]]\nfalse\n[:aliased, :aliased_again, :first, :inherited_method]\nnil\ntrue\n";
    let output = run_example("oop/method_added_hook_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_method_defined_visibility_execution() {
    let expected = "public_mixin true true false false\nprotected_mixin true false true false\nprivate_mixin false false false true\npublic_holder true true false false\nprivate_holder false false false true\n[:private_mixin]\n[:protected_mixin]\ntrue\nfalse\n42 is not a symbol nor a string\n";
    let output = run_example("oop/method_defined_visibility.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_method_defined_visibility_parens_execution() {
    let expected = "public_mixin true true false false\nprotected_mixin true false true false\nprivate_mixin false false false true\npublic_holder true true false false\nprivate_holder false false false true\n[:private_mixin]\n[:protected_mixin]\ntrue\nfalse\n42 is not a symbol nor a string\n";
    let output = run_example("oop/method_defined_visibility_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_method_removed_hook_execution() {
    let expected = "[[:removed, :doomed], [:undefined, :shadowed]]\n[]\nnil\ntrue\ncan\'t modify frozen Module: the frozen module\n";
    let output = run_example("oop/method_removed_hook.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_method_removed_hook_parens_execution() {
    let expected = "[[:removed, :doomed], [:undefined, :shadowed]]\n[]\nnil\ntrue\ncan\'t modify frozen Module: the frozen module\n";
    let output = run_example("oop/method_removed_hook_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_remove_class_variable_execution() {
    let expected = ":shared\nfalse\n:own\n@@shared: NameError\n@shared: NameError\nshared: NameError\n@@absent: NameError\nfalse\n:no_block\n";
    let output = run_example("oop/remove_class_variable.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_remove_class_variable_parens_execution() {
    let expected = ":shared\nfalse\n:own\n@@shared: NameError\n@shared: NameError\nshared: NameError\n@@absent: NameError\nfalse\n:no_block\n";
    let output = run_example("oop/remove_class_variable_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_remove_const_execution() {
    let expected = ":doomed\n:also\n[:KEPT]\nname: NameError\n__CONSTX__: NameError\n@Name: NameError\nName=: NameError\nMissing: NameError\ninherited: NameError\nnil\ntrue\ntrue\n";
    let output = run_example("oop/remove_const.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_remove_const_parens_execution() {
    let expected = ":doomed\n:also\n[:KEPT]\nname: NameError\n__CONSTX__: NameError\n@Name: NameError\nName=: NameError\nMissing: NameError\ninherited: NameError\nnil\ntrue\ntrue\n";
    let output = run_example("oop/remove_const_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_remove_method_execution() {
    let expected = "Child\n[]\n:parent\ninherited: NameError\nmissing: NameError\nChild\nfrozen: FrozenError\ntrue\n-1\ntrue\n";
    let output = run_example("oop/remove_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_remove_method_parens_execution() {
    let expected = "Child\n[]\n:parent\ninherited: NameError\nmissing: NameError\nChild\nfrozen: FrozenError\ntrue\n-1\ntrue\n";
    let output = run_example("oop/remove_method_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_undef_method_execution() {
    let expected = "Child\nfalse\n:parent\nundefined method \'never_defined\' for class \'Child\'\nundefined method \'not_exist\' for class \'String\'\nChild\nfrozen: FrozenError\n/Hello World/\ntrue\n\"a\\\\.b\\\\*c\"\n";
    let output = run_example("oop/undef_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_undef_method_parens_execution() {
    let expected = "Child\nfalse\n:parent\nundefined method \'never_defined\' for class \'Child\'\nundefined method \'not_exist\' for class \'String\'\nChild\nfrozen: FrozenError\n/Hello World/\ntrue\n\"a\\\\.b\\\\*c\"\n";
    let output = run_example("oop/undef_method_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_undefined_instance_methods_execution() {
    let expected = "[:retired]\n[:from_module, :kept, :own]\n[]\nfalse\nnil\n";
    let output = run_example("oop/undefined_instance_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_undefined_instance_methods_parens_execution() {
    let expected = "[:retired]\n[:from_module, :kept, :own]\n[]\nfalse\nnil\n";
    let output = run_example("oop/undefined_instance_methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_undef_keyword_execution() {
    let expected = "hello\nfalse\nfalse\ngreet undefined\n";
    let output = run_example("oop/undef_keyword.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_send_dispatch_execution() {
    let expected = concat!(
        "hello\nhello\nhello\n",
        "[:one]\n",
        "[:one, :two]\n",
        "42 is not a symbol nor a string\n",
        "no method name given\n",
        "NoMethodError for a name nothing defines\n",
        "[:!, :!=, :==, :__id__, :__send__, :equal?, :instance_eval, :instance_exec]\n",
        "true\n",
        "[:first, :second]\n"
    );
    let output = run_example("oop/send/dispatch.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_send_dispatch_parens_execution() {
    let expected = concat!(
        "hello\nhello\nhello\n",
        "[:one]\n",
        "[:one, :two]\n",
        "42 is not a symbol nor a string\n",
        "no method name given\n",
        "NoMethodError for a name nothing defines\n",
        "[:!, :!=, :==, :__id__, :__send__, :equal?, :instance_eval, :instance_exec]\n",
        "true\n",
        "[:first, :second]\n"
    );
    let output = run_example("oop/send/dispatch_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn method_added_hears_of_methods_defined_in_class_exec() {
    let output = run_example("oop/method_added_for_class_exec_definitions.rb");
    assert_eq!(output, "added receive\nadded allow\nadded stub\n");
}

#[test]
fn a_subclass_reaches_a_class_method_its_parents_singleton_class_redefines() {
    let output = run_example("oop/subclass_reaches_parents_redefined_singleton_method.rb");
    assert_eq!(output, "\"Hello\"\n\"Howdy\"\n\"Hello\"\nnil\n");
}

#[test]
fn test_a_class_answers_case_equality_with_its_own_method() {
    let output = run_example("oop/a_class_with_its_own_case_equality.rb");
    assert_eq!(
        output,
        concat!("[true, true, false]\n", "false\n", ":boolean\n",)
    );
}

#[test]
fn test_aliases_of_native_methods_answer_for_nil_true_and_false() {
    let output = run_example("oop/aliases_of_native_methods_on_nil_true_and_false.rb");
    assert_eq!(
        output,
        "true\ntrue\n[\"nil\", \"true\", \"false\"]\n\"nil\"\n\"nil\"\n"
    );
}

#[test]
fn test_an_array_subclass_answers_an_alias_named_after_another_native_method() {
    let output = run_example("oop/array_subclass_aliases_to_s_to_join.rb");
    assert_eq!(
        output,
        concat!(
            "\"<p>text</p>\"\n",
            "<p>text</p>\n",
            "\"<p>text</p>\"\n",
            "\"<p>text</p>\"\n",
            "\"ab\"\n",
        )
    );
}

#[test]
fn test_a_bracket_against_a_bare_super_indexes_its_answer() {
    let output = run_example("oop/super_followed_by_an_index.rb");
    assert_eq!(output, "\"#<Base value:42>\"\n5\n2\n");
}

#[test]
fn test_include_leaves_a_module_a_superclass_has_where_the_superclass_has_it() {
    let output = run_example("oop/include_skips_a_module_a_superclass_has.rb");
    assert_eq!(
        output,
        concat!(
            "[Single, Comparable, Serial, Real, Abstract, Service]\n",
            ":real\n",
            "true\n",
            "[Comparable, Serial, Service, Kernel]\n",
        )
    );
}

#[test]
fn test_class_method_super_goes_past_a_module_a_subclass_extended() {
    let output = run_example("oop/class_method_super_past_a_module_a_subclass_extended.rb");
    assert_eq!(
        output,
        concat!(
            "[:safe, :mid, :root]\n",
            "[:own_child, :safe, :mid, :root]\n",
            "checked Value\n",
            "barrier\n",
            "1\n",
        )
    );
}

#[test]
fn test_a_public_override_holds_for_an_object_with_a_singleton_class() {
    let output = run_example("oop/public_override_holds_after_a_singleton_class_is_made.rb");
    assert_eq!(
        output,
        concat!(
            "[:unscheduled]\n",
            "[:pending]\n",
            "protected method 'secret' called for #<Guarded:0x>\n",
            "private method 'hidden' called for #<Guarded:0x>\n",
        )
    );
}

#[test]
fn test_clone_has_the_singleton_methods_while_initialize_copy_runs() {
    let output = run_example("oop/clone_has_singleton_methods_during_initialize_copy.rb");
    assert_eq!(output, "[:stubbed]\n[:synchronize]\n[:stubbed]\n");
}
