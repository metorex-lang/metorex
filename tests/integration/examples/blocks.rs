use super::run_example;

#[test]
fn test_blocks_yield_arguments_before_the_block_check_execution() {
    let output = run_example("blocks/yield_arguments_before_the_block_check.rb");
    assert_eq!(
        output,
        "ArgumentError\n\"no block given (yield)\"\n"
    );
}

#[test]
fn test_blocks_block_writes_a_local_from_a_branch_not_taken_execution() {
    let output = run_example("blocks/block_writes_a_local_from_a_branch_not_taken.rb");
    assert_eq!(
        output,
        "[1, :loaded]\n[3, :queried]\nnil\n[2, 3]\n[4, 4]\n"
    );
}

#[test]
fn test_blocks_yield_then_shift_operator_execution() {
    let output = run_example("blocks/yield_then_shift_operator.rb");
    assert_eq!(output, "\"<form>body</form>\"\n5\n");
}
