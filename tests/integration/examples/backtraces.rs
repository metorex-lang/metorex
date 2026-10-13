// Where a frame says it is, for blocks run from another file.

use super::run_example;

#[test]
fn test_blocks_report_the_file_they_were_written_in() {
    let expected = "call: block_file_test.rb:7\n\
                    instance_exec: block_file_test.rb:10\n\
                    raised: block_file_test.rb:14:in 'block in <main>'\n\
                    module_exec: block_file_test.rb:19\n\
                    class_exec: block_file_test.rb:22\n\
                    module_exec raised: block_file_test.rb:26:in 'block in <main>'\n";
    let output = run_example("backtraces/block_file_test.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_a_chained_call_is_on_the_line_of_its_method_name() {
    let output = run_example("backtraces/chained_call_line.rb");
    assert_eq!(output, "14\n20\n24\n39\n45\n");
}

#[test]
fn test_a_method_s_frame_names_its_own_file_for_a_dispatch_error() {
    let output = run_example("backtraces/private_call_frame/main.rb");
    assert_eq!(
        output,
        "private method 'hidden' called for class Registry\nlibrary.rb:13:in 'Settings#alias_name'\nmain.rb:4:in '<main>'\nlibrary.rb:17:in 'Settings#missing_name'\nmain.rb:11:in '<main>'\n"
    );
}

#[test]
fn test_an_at_exit_handler_s_backtrace_ends_at_its_block() {
    let output = run_example("backtraces/at_exit_handler_raise.rb");
    assert_eq!(
        output,
        "at_exit_handler_raise.rb:1:in 'Object#boom'\nat_exit_handler_raise.rb:3:in 'block in <main>'\n"
    );
}
