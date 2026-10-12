// Examples covering Data value objects

use super::run_example;

/// The expected output of both `data_objects/value_objects` variants, which
/// differ only in whether the calls are written with parentheses.
const VALUE_OBJECTS_OUTPUT: &str = "[:amount, :unit]\n42\n\"km\"\n{amount: 3, unit: \"mi\"}\ntrue\n\"#<data Measure amount=1, unit=\\\"m\\\">\"\ntrue\ntrue\ntrue\nfalse\ntrue\n{amount: 42, unit: \"m\"}\n[42, \"km\"]\n{amount: 42}\n{amount: 42, unit: \"km\"}\n{\"amount\" => 42, \"unit\" => \"km\"}\n\"Rain (1999)\"\n\"missing keyword: :unit\"\n\"unknown keyword: :system\"\n";

#[test]
fn test_data_objects_value_objects_execution() {
    let output = run_example("data_objects/value_objects.rb");
    assert_eq!(output, VALUE_OBJECTS_OUTPUT);
}

#[test]
fn test_data_objects_value_objects_no_parens_execution() {
    let output = run_example("data_objects/value_objects_no_parens.rb");
    assert_eq!(output, VALUE_OBJECTS_OUTPUT);
}

/// The expected output of both `data_objects/data_member_keys` variants.
const DATA_MEMBER_KEYS_OUTPUT: &str = "#<data Reading meter=\"M-12\", value=40>\n#<data Reading meter=\"M-12\", value=41>\n\"3 is not a symbol nor a string\"\n\"unknown keyword: \\\"unit\\\"\"\n{\"meter\" => \"M-12\"}\n#<data Labeled name=\"PUMP\">\n{\"zone\" => 4, 7 => :seven}\n";

#[test]
fn test_data_objects_data_member_keys_execution() {
    let output = run_example("data_objects/data_member_keys.rb");
    assert_eq!(output, DATA_MEMBER_KEYS_OUTPUT);
}

#[test]
fn test_data_objects_data_member_keys_no_parens_execution() {
    let output = run_example("data_objects/data_member_keys_no_parens.rb");
    assert_eq!(output, DATA_MEMBER_KEYS_OUTPUT);
}
