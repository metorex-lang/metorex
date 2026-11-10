use crate::common::EXAMPLES_DIR;
use std::process::Command;

use super::run_example;

#[test]
fn test_stdlib_strings_execution() {
    let expected = "11\n11\nHELLO WORLD\nhello world\ndlrow olleh\nhello\nhello\n2\nhello\nworld\none, two, three\nonetwothree\nhello\nworld\nworld\ntrue\nfalse\ntrue\ntrue\nfalse\ntrue\nfalse\n";
    let output = run_example("stdlib/string/strings.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_strings_parens_execution() {
    let expected = "11\n11\nHELLO WORLD\nhello world\ndlrow olleh\nhello\nhello\n2\nhello\nworld\none, two, three\nonetwothree\nhello\nworld\nworld\ntrue\nfalse\ntrue\ntrue\nfalse\ntrue\nfalse\n";
    let output = run_example("stdlib/string/strings_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_arrays_execution() {
    let expected = "8\n8\n1\n2\n3\n4\n4\n1\n2\n3\n1\n2\n3\n0\n2\n3\n1\n1\n2\n3\n4\n5\n6\n9\n6\n2\n9\n5\n1\n4\n1\n3\n2\n4\n6\n2\n4\n6\n15\na, b, c\nabc\n";
    let output = run_example("stdlib/array/arrays.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_arrays_parens_execution() {
    let expected = "8\n8\n1\n2\n3\n4\n4\n1\n2\n3\n1\n2\n3\n0\n2\n3\n1\n1\n2\n3\n4\n5\n6\n9\n6\n2\n9\n5\n1\n4\n1\n3\n2\n4\n6\n2\n4\n6\n15\na, b, c\nabc\n";
    let output = run_example("stdlib/array/arrays_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_hashes_execution() {
    let expected = "alice\nbob\ncharlie\n25\n30\n35\ntrue\nfalse\n3\n30\n0\n25\n4\n90\n3\n";
    let output = run_example("stdlib/hash/hashes.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_hashes_parens_execution() {
    let expected = "alice\nbob\ncharlie\n25\n30\n35\ntrue\nfalse\n3\n30\n0\n25\n4\n90\n3\n";
    let output = run_example("stdlib/hash/hashes_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_numbers_execution() {
    let expected = "42\n7\n42.0\n42\n42\n10\n3.14\n2.5\n4\n3\n3.14\n3\n3.14\n3.14\n";
    let output = run_example("stdlib/numbers/numbers.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_numbers_parens_execution() {
    let expected = "42\n7\n42.0\n42\n42\n10\n3.14\n2.5\n4\n3\n3.14\n3\n3.14\n3.14\n";
    let output = run_example("stdlib/numbers/numbers_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_io_execution() {
    let expected = "Hello from puts\nHello from print\n42\nHello from file!\ntrue\n";
    let output = run_example("stdlib/io/io.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_io_parens_execution() {
    let expected = "Hello from puts\nHello from print\n42\nHello from file!\ntrue\n";
    let output = run_example("stdlib/io/io_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_sets_execution() {
    let expected = "3\n3\ntrue\nfalse\n2\ntrue\nfalse\n6\n2\n2\n3\n3\n";
    let output = run_example("stdlib/sets/sets.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_sets_parens_execution() {
    let expected = "3\n3\ntrue\nfalse\n2\ntrue\nfalse\n6\n2\n2\n3\n3\n";
    let output = run_example("stdlib/sets/sets_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_testing_framework_execution() {
    let expected = "\x1b[1mMath operations\x1b[0m\n  \x1b[32mPASS\x1b[0m: adds numbers\n  \x1b[32mPASS\x1b[0m: multiplies numbers\n  \x1b[32mPASS\x1b[0m: divides numbers\n\x1b[32m3 passed\x1b[0m, 0 failed\n\x1b[1mString operations\x1b[0m\n  \x1b[32mPASS\x1b[0m: concatenates strings\n  \x1b[32mPASS\x1b[0m: gets length\n\x1b[32m2 passed\x1b[0m, 0 failed\n\x1b[1mType checking\x1b[0m\n  \x1b[32mPASS\x1b[0m: checks integer type\n  \x1b[32mPASS\x1b[0m: checks truthiness\n  \x1b[32mPASS\x1b[0m: checks nil\n\x1b[32m3 passed\x1b[0m, 0 failed\n\x1b[1mAssertions\x1b[0m\n  \x1b[32mPASS\x1b[0m: assert_equal catches mismatches\n  \x1b[32mPASS\x1b[0m: assert catches false\n\x1b[32m2 passed\x1b[0m, 0 failed\n\x1b[1mFiltered suite\x1b[0m\n  \x1b[32mPASS\x1b[0m: add test\n\x1b[32m1 passed\x1b[0m, 0 failed\n";
    let output = run_example("stdlib/testing/framework.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_testing_framework_parens_execution() {
    let expected = "\x1b[1mMath operations\x1b[0m\n  \x1b[32mPASS\x1b[0m: adds numbers\n  \x1b[32mPASS\x1b[0m: multiplies numbers\n  \x1b[32mPASS\x1b[0m: divides numbers\n\x1b[32m3 passed\x1b[0m, 0 failed\n\x1b[1mString operations\x1b[0m\n  \x1b[32mPASS\x1b[0m: concatenates strings\n  \x1b[32mPASS\x1b[0m: gets length\n\x1b[32m2 passed\x1b[0m, 0 failed\n\x1b[1mType checking\x1b[0m\n  \x1b[32mPASS\x1b[0m: checks integer type\n  \x1b[32mPASS\x1b[0m: checks truthiness\n  \x1b[32mPASS\x1b[0m: checks nil\n\x1b[32m3 passed\x1b[0m, 0 failed\n\x1b[1mAssertions\x1b[0m\n  \x1b[32mPASS\x1b[0m: assert_equal catches mismatches\n  \x1b[32mPASS\x1b[0m: assert catches false\n\x1b[32m2 passed\x1b[0m, 0 failed\n\x1b[1mFiltered suite\x1b[0m\n  \x1b[32mPASS\x1b[0m: add test\n\x1b[32m1 passed\x1b[0m, 0 failed\n";
    let output = run_example("stdlib/testing/framework_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_type_introspection_execution() {
    let expected = "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nNumeric\nBasicObject\n6\ntrue\ntrue\nAnimal\n2\nRex\n3\n4\n";
    let output = run_example("builtins/type_introspection.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_type_introspection_parens_execution() {
    let expected = "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nNumeric\nBasicObject\n6\ntrue\ntrue\nAnimal\n2\nRex\n3\n4\n";
    let output = run_example("builtins/type_introspection_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_execution() {
    let expected = "hello world\nnum: 42\npi: 3.14\nhex: ff\ncart has 5 items\n100% complete: done\n\"test\"\n00042\nleft      |\n";
    let output = run_example("stdlib/string/format.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_parens_execution() {
    let expected = "hello world\nnum: 42\npi: 3.14\nhex: ff\ncart has 5 items\n100% complete: done\n\"test\"\n00042\nleft      |\n";
    let output = run_example("stdlib/string/format_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_extended_execution() {
    let expected = "42\n+42\n-5\n 42\n-5\n+3.140000\n 3.140000\n3\n+3\n 3\nFF\n10\n1010\nA\nh\nnil\n42\nhel\n     right|\n0000000042\n100%\n";
    let output = run_example("stdlib/string/format_extended.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_extended_parens_execution() {
    let expected = "42\n+42\n-5\n 42\n-5\n+3.140000\n 3.140000\n3\n+3\n 3\nFF\n10\n1010\nA\nh\nnil\n42\nhel\n     right|\n0000000042\n100%\n";
    let output = run_example("stdlib/string/format_extended_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_contexts_execution() {
    let expected = "(?-mix:hello)\n(?-mix:foo)\n(?i-mx:bar)\n(?-mix:[a-z]+)\n(?-mix:path\\/to\\/file)\n5\n4\n5\n";
    let output = run_example("stdlib/regex/contexts.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_contexts_parens_execution() {
    let expected = "(?-mix:hello)\n(?-mix:foo)\n(?i-mx:bar)\n(?-mix:[a-z]+)\n(?-mix:path\\/to\\/file)\n5\n4\n5\n";
    let output = run_example("stdlib/regex/contexts_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_literals_execution() {
    let expected = "(?-mix:hello)\n(?i-mx:world)\n5\n(?-mix:[a-z]+\\d+)\n(?-mix:hello\\/world)\n";
    let output = run_example("stdlib/regex/literals.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_literals_parens_execution() {
    let expected = "(?-mix:hello)\n(?i-mx:world)\n5\n(?-mix:[a-z]+\\d+)\n(?-mix:hello\\/world)\n";
    let output = run_example("stdlib/regex/literals_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_slice_edge_execution() {
    let expected = "ell\nhe\nll\nnil\n\"\"\ndone\nfoobar\n";
    let output = run_example("stdlib/string/slice_edge.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_slice_edge_parens_execution() {
    let expected = "ell\nhe\nll\nnil\n\"\"\ndone\nfoobar\n";
    let output = run_example("stdlib/string/slice_edge_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_edge_execution() {
    let expected = concat!(
        "c4\n",
        "C4\n",
        "127\n",
        "1010\n",
        "invalid value for Integer(): \"hello\"\n",
        "A\n",
        "Z\n",
        "      hi!\n",
        "7\n",
        "5.000000\n",
        "5.00\n",
        "test%\n",
    );
    let output = run_example("stdlib/string/format_edge.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_edge_parens_execution() {
    let expected = concat!(
        "c4\n",
        "C4\n",
        "127\n",
        "1010\n",
        "invalid value for Integer(): \"hello\"\n",
        "A\n",
        "Z\n",
        "      hi!\n",
        "7\n",
        "5.000000\n",
        "5.00\n",
        "test%\n",
    );
    let output = run_example("stdlib/string/format_edge_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_to_i_execution() {
    let expected = "42\n99\n0\n-7\n3\n3.14\n0.0\nworld\nherro\nherlo\ntrue\nfalse\n";
    let output = run_example("stdlib/string/to_i.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_to_i_parens_execution() {
    let expected = "42\n99\n0\n-7\n3\n3.14\n0.0\nworld\nherro\nherlo\ntrue\nfalse\n";
    let output = run_example("stdlib/string/to_i_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_extended_execution() {
    let expected =
        "15\n25\n5\n6\n1\n2\n3\n1\n2\n3\n4\n5\n3\n1\n2\n1\n8\ntrue\nfalse\n10\n30\ntrue\nfalse\n";
    let output = run_example("stdlib/array/extended.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_extended_parens_execution() {
    let expected =
        "15\n25\n5\n6\n1\n2\n3\n1\n2\n3\n4\n5\n3\n1\n2\n1\n8\ntrue\nfalse\n10\n30\ntrue\nfalse\n";
    let output = run_example("stdlib/array/extended_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_new_methods_execution() {
    let expected = "15\n25\n12345\n3\n4\n3\n1\n2\n3\n4\n1\n2\n3\ntrue\nfalse\n10\n30\n\n\ntrue\nfalse\ntrue\n1\n8\n1.2\n3.5\n\n\n3\n1\n2\n";
    let output = run_example("stdlib/array/new_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_new_methods_parens_execution() {
    let expected =
        "15\n25\n3\n4\n1\n2\n3\n4\n1\n2\n3\ntrue\nfalse\n10\n30\ntrue\nfalse\n1\n8\n3\n1\n2\n";
    let output = run_example("stdlib/array/new_methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_new_methods_execution() {
    let expected =
        "42\n99\n0\n-7\n3\n3.14\n0.0\n-2.5\noriginal\nhell0 w0rld\nbbbbbb\nhi hello\ntrue\nfalse\n";
    let output = run_example("stdlib/string/new_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_new_methods_parens_execution() {
    let expected =
        "42\n99\n0\n-7\n3\n3.14\n0.0\n-2.5\noriginal\nhell0 w0rld\nbbbbbb\nhi hello\ntrue\nfalse\n";
    let output = run_example("stdlib/string/new_methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_error_paths_test_execution() {
    let expected = "3.14\n3.14\n4\n3\n4\n3\n3.14\n42.0\n42\n42\n2\nell\ntrue\ntrue\ntrue\nhello\nhello\nHELLO\nolleh\n1, 2, 3\n2, 1, 3\n3\n0\n0\n123\n\nfalse\ntrue\n3\n15\n3\nerror_paths_test passed\n";
    let output = run_example("stdlib/error_paths_test.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_functional_execution() {
    let expected = "true\nfalse\ntrue\nfalse\ntrue\nfalse\n2\n2\n4\n2\n3\n15\n120\n";
    let output = run_example("stdlib/array/functional.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_functional_parens_execution() {
    let expected = "true\nfalse\ntrue\nfalse\ntrue\nfalse\n2\n2\n4\n2\n3\n15\n120\n";
    let output = run_example("stdlib/array/functional_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_hash_shorthand_execution() {
    let expected = "Alice\n30\nlocalhost\n8080\ntrue\n3\n2\n";
    let output = run_example("stdlib/hash/shorthand.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_hash_shorthand_parens_execution() {
    let expected = "Alice\n30\nlocalhost\n8080\ntrue\n3\n2\n";
    let output = run_example("stdlib/hash/shorthand_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_file_and_string_reads_execution() {
    let expected = "[\"alpha\", \"beta\"]\n2\n11\n\"beta\"\nnil\n";
    let output = run_example("stdlib/file_and_string_reads.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_file_and_string_reads_parens_execution() {
    let expected = "[\"alpha\", \"beta\"]\n2\n11\n\"beta\"\nnil\n";
    let output = run_example("stdlib/file_and_string_reads_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_io_popen_execution() {
    let expected = concat!(
        "hello\n", "true\n", "true\n", "false\n", "true\n", "true\n", "0\n", "false\n", "nil\n",
        "true\n", "0\n", "one\n", "err\n", "3\n", "false\n",
    );
    let output = run_example("runtime/io_popen.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_io_popen_parens_execution() {
    let expected = concat!(
        "hello\n", "true\n", "true\n", "false\n", "true\n", "true\n", "0\n", "false\n", "nil\n",
        "true\n", "0\n", "one\n", "err\n", "3\n", "false\n",
    );
    let output = run_example("runtime/io_popen_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_dump_execution() {
    let expected = concat!(
        "\"plain\"\n",
        "\"with \\\"quotes\\\"\"\n",
        "\"tab\\there\"\n",
        "\"line\\nbreak\"\n",
        "\"back\\\\slash\"\n",
        "\"interp \\#{x} and \\#@ivar\"\n",
        "\"caf\\u00E9\"\n",
    );
    let output = run_example("stdlib/string/dump.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_dump_parens_execution() {
    let expected = concat!(
        "\"plain\"\n",
        "\"with \\\"quotes\\\"\"\n",
        "\"tab\\there\"\n",
        "\"line\\nbreak\"\n",
        "\"back\\\\slash\"\n",
        "\"interp \\#{x} and \\#@ivar\"\n",
        "\"caf\\u00E9\"\n",
    );
    let output = run_example("stdlib/string/dump_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_dir_chdir_execution() {
    let expected = "true\ntrue\ntrue\ntrue\ntrue\n42\n";
    let output = run_example("runtime/dir_chdir.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_dir_chdir_parens_execution() {
    let expected = "true\ntrue\ntrue\ntrue\ntrue\n42\n";
    let output = run_example("runtime/dir_chdir_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_magic_dir_execution() {
    let expected = "true\ntrue\n\".\"\n\"foo\"\nnil\n";
    let output = run_example("runtime/magic_dir.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_magic_dir_parens_execution() {
    let expected = "true\ntrue\n\".\"\n\"foo\"\nnil\n";
    let output = run_example("runtime/magic_dir_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_at_exit_handlers_execution() {
    let expected = concat!(
        "called without a block\n",
        "true\n",
        "main body\n",
        "registered last\n",
        "outer\n",
        "outer done\n",
        "nested\n",
        "registered first\n",
    );
    let output = run_example("runtime/at_exit_handlers.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_at_exit_handlers_parens_execution() {
    let expected = concat!(
        "called without a block\n",
        "true\n",
        "main body\n",
        "registered last\n",
        "outer\n",
        "outer done\n",
        "nested\n",
        "registered first\n",
    );
    let output = run_example("runtime/at_exit_handlers_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_at_exit_exit_status_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/runtime/at_exit_exit_status.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");

    assert_eq!(stdout, "main body\nfirst handler\nlast handler\n");
    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn test_runtime_at_exit_last_exception_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/runtime/at_exit_last_exception.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");

    assert_eq!(stdout, "RuntimeError\nboom\n");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_runtime_backtick_command_execution() {
    let expected = concat!(
        "\"hello world\\n\"\n",
        "0\n",
        "true\n",
        "true\n",
        "false\n",
        "Process::Status\n",
        "7\n",
        "false\n",
        "\"through the module\\n\"\n",
        "\"coerced\\n\"\n",
        "No such file or directory - nonexistent_command_xyz\n",
        "true\n",
        ":`\n"
    );
    let output = run_example("runtime/backtick_command.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_backtick_command_parens_execution() {
    let expected = concat!(
        "\"hello world\\n\"\n",
        "0\n",
        "true\n",
        "true\n",
        "false\n",
        "Process::Status\n",
        "7\n",
        "false\n",
        "\"through the module\\n\"\n",
        "\"coerced\\n\"\n",
        "No such file or directory - nonexistent_command_xyz\n",
        "true\n",
        ":`\n"
    );
    let output = run_example("runtime/backtick_command_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_dash_n_chomp_execution() {
    let expected = concat!(
        "abc\n",
        "abc\n",
        "abc\n",
        "\"abc\\n\"\n",
        "ab\n",
        "\"abc\"\n",
        "abc\n",
        "true\n",
        "true\n",
    );
    let output = run_example("runtime/dash_n_chomp.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_dash_n_chomp_parens_execution() {
    let expected = concat!(
        "abc\n",
        "abc\n",
        "abc\n",
        "\"abc\\n\"\n",
        "ab\n",
        "\"abc\"\n",
        "abc\n",
        "true\n",
        "true\n",
    );
    let output = run_example("runtime/dash_n_chomp_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_io_popen_argv_execution() {
    let expected = "got: a line\nno input\n0\n";
    let output = run_example("runtime/io_popen_argv.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_io_popen_argv_parens_execution() {
    let expected = "got: a line\nno input\n0\n";
    let output = run_example("runtime/io_popen_argv_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_exec_replaces_process_execution() {
    let expected = "before exec\nreplaced\n";
    let output = run_example("runtime/exec_replaces_process.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_exec_replaces_process_parens_execution() {
    let expected = "before exec\nreplaced\n";
    let output = run_example("runtime/exec_replaces_process_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_exec_missing_command_execution() {
    let expected = "true\nNo such file or directory - definitely_not_a_command_xyz\nErrno::EPIPE\n";
    let output = run_example("runtime/exec_missing_command.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_exec_missing_command_parens_execution() {
    let expected = "true\nNo such file or directory - definitely_not_a_command_xyz\nErrno::EPIPE\n";
    let output = run_example("runtime/exec_missing_command_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_exit_bang_skips_handlers_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/runtime/exit_bang_skips_handlers.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");

    assert_eq!(stdout, "before\n");
    assert_eq!(output.status.code(), Some(21));
}

#[test]
fn test_runtime_at_exit_overrides_error_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/runtime/at_exit_overrides_error.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");

    assert_eq!(stdout, "in at_exit\n$! is RuntimeError:original error\n");
    assert_eq!(output.status.code(), Some(21));
}

#[test]
fn test_runtime_fork_child_process_execution() {
    let expected = "42\n0\n7\nwritten by child\ntrue\ntrue\nfalse\nfalse\ntrue\n";
    let output = run_example("runtime/fork_child_process.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_fork_child_process_parens_execution() {
    let expected = "42\n0\n7\nwritten by child\ntrue\ntrue\nfalse\nfalse\ntrue\n";
    let output = run_example("runtime/fork_child_process_parens.rb");
    assert_eq!(output, expected);
}

/// The expected output of both `runtime/forked_popen` variants.
const FORKED_POPEN_OUTPUT: &str = concat!(
    "\"hello from child\\n\"\n",
    "true\n",
    "false\n",
    "\"child read \\\"to child\\\\n\\\"\\n\"\n",
    "3\n",
    "\"from the child's block\\n\"\n",
    "IOError\n",
    "true\n",
    "true\n",
    "5\n",
    "\"I'm shared!\"\n",
    "\"I'm shared!\"\n",
    "1\n"
);

#[test]
fn test_runtime_forked_popen_execution() {
    let output = run_example("runtime/forked_popen.rb");
    assert_eq!(output, FORKED_POPEN_OUTPUT);
}

#[test]
fn test_runtime_forked_popen_parens_execution() {
    let output = run_example("runtime/forked_popen_parens.rb");
    assert_eq!(output, FORKED_POPEN_OUTPUT);
}

#[test]
fn test_stdlib_string_format_keywords_execution() {
    let expected = concat!(
        "a and b\n",
        "test value\n",
        "hello, world!\n",
        "00042\n",
        "3.14\n",
        "through the module\n",
        "key{missing} not found\n",
        "true\n",
        "true\n",
    );
    let output = run_example("stdlib/string/format_keywords.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_keywords_parens_execution() {
    let expected = concat!(
        "a and b\n",
        "test value\n",
        "hello, world!\n",
        "00042\n",
        "3.14\n",
        "through the module\n",
        "key{missing} not found\n",
        "true\n",
        "true\n",
    );
    let output = run_example("stdlib/string/format_keywords_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_verbose_warning_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/stdlib/string/format_verbose_warning.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr was not utf8");

    assert_eq!(stdout, "no placeholders\nstill quiet\n");
    assert_eq!(
        stderr,
        format!("{full_path}:2: warning: too many arguments for format string\n")
    );
}

#[test]
fn test_runtime_stdout_redirect_execution() {
    let expected = concat!(
        "\"\\\"captured\"\n",
        "\"\nand this\n",
        "[<described>, \"text\", :symbol, nil]\n",
        "<described>\n",
    );
    let output = run_example("runtime/stdout_redirect.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_stdout_redirect_parens_execution() {
    let expected = concat!(
        "\"\\\"captured\"\n",
        "\"\nand this\n",
        "[<described>, \"text\", :symbol, nil]\n",
        "<described>\n",
    );
    let output = run_example("runtime/stdout_redirect_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_stringio_printf_execution() {
    let expected = concat!(
        "\"start: value-7 shovelled printed\\n\"\n",
        "\"first\\n\"\n",
        "\"second\\n\"\n",
        "\"\"\n",
        "\"first\"\n",
        "one and two\n",
        "42\n",
        "false\n",
        "true\n",
        "written\n"
    );
    let output = run_example("stdlib/string/stringio_printf.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_stringio_printf_parens_execution() {
    let expected = concat!(
        "\"start: value-7 shovelled printed\\n\"\n",
        "\"first\\n\"\n",
        "\"second\\n\"\n",
        "\"\"\n",
        "\"first\"\n",
        "one and two\n",
        "42\n",
        "false\n",
        "true\n",
        "written\n"
    );
    let output = run_example("stdlib/string/stringio_printf_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_math_functions_execution() {
    let expected = concat!(
        "2.0\n",
        "3.0\n",
        "5.0\n",
        "0.0\n",
        "3.0\n",
        "3.0\n",
        "3.0\n",
        "1.0\n",
        "[0.6025390625, 11]\n",
        "1234.0\n",
        "5.0\n",
        "true\n",
        "10001.0\n",
        "Numerical argument is out of domain - \"sqrt\"\n",
        "can't convert String into Float\n",
        "true\n"
    );
    let output = run_example("stdlib/math/functions.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_math_functions_parens_execution() {
    let expected = concat!(
        "2.0\n",
        "3.0\n",
        "5.0\n",
        "0.0\n",
        "3.0\n",
        "3.0\n",
        "3.0\n",
        "1.0\n",
        "[0.6025390625, 11]\n",
        "1234.0\n",
        "5.0\n",
        "true\n",
        "10001.0\n",
        "Numerical argument is out of domain - \"sqrt\"\n",
        "can't convert String into Float\n",
        "true\n"
    );
    let output = run_example("stdlib/math/functions_parens.rb");
    assert_eq!(output, expected);
}

/// The expected output of both `stdlib/string/character_sets` variants.
const STRING_CHARACTER_SETS_OUTPUT: &str = "3\n1\n3\n\"heo word\"\n\"abc\"\n\"abbbccc\"\n\"hippo\"\n\"ifmmp\"\n\"*e**o\"\n\"hero\"\n\"invalid range \\\"h-e\\\" in string transliteration\"\n\"hello\"\n\"hello\"\n\"llo\"\n\"hel\"\n[\"he\", \"l\", \"lo\"]\n[\"hel\", \"l\", \"o\"]\n[\"he\", \"ll\", \"o\"]\n:hello\n294\n6\n\"   hi    \"\n\"121hi1212\"\n0\ntrue\nnil\n\"x\"\nnil\n[\"a\", \"b\", \"c\", \"d\", \"e\"]\n[\"9\", \":\", \";\", \"<\", \"=\", \">\", \"?\", \"@\", \"A\"]\n[\"8\", \"9\", \"10\", \"11\"]\n[\"a\", \"b\", \"c\"]\n[]\n";

#[test]
fn test_stdlib_string_character_sets_execution() {
    let output = run_example("stdlib/string/character_sets.rb");
    assert_eq!(output, STRING_CHARACTER_SETS_OUTPUT);
}

#[test]
fn test_stdlib_string_character_sets_no_parens_execution() {
    let output = run_example("stdlib/string/character_sets_no_parens.rb");
    assert_eq!(output, STRING_CHARACTER_SETS_OUTPUT);
}

/// The expected output of both `stdlib/dir/reading_a_directory` variants.
const READING_A_DIRECTORY_OUTPUT: &str = "true\nfalse\nfalse\n[\".\", \"..\", \"held.txt\"]\n[\"held.txt\"]\n[\".\", \"..\", \"held.txt\"]\n[\"held.txt\"]\nEnumerator\nnil\nErrno::ENOTEMPTY\nErrno::ENOENT\n0\nfalse\ntrue\n";

#[test]
fn test_stdlib_dir_reading_a_directory_execution() {
    let output = run_example("stdlib/dir/reading_a_directory.rb");
    assert_eq!(output, READING_A_DIRECTORY_OUTPUT);
}

#[test]
fn test_stdlib_dir_reading_a_directory_no_parens_execution() {
    let output = run_example("stdlib/dir/reading_a_directory_no_parens.rb");
    assert_eq!(output, READING_A_DIRECTORY_OUTPUT);
}

/// The expected output of both `stdlib/marshal_round_trip` variants, which
/// differ only in whether the calls are written with parentheses.
const MARSHAL_ROUND_TRIP_OUTPUT: &str = concat!(
    "[4, 8, 102, 8, 48, 46, 53]\n",
    "[4, 8, 105, 47]\n",
    "[1, \"two\", :three, {four: 4}]\n",
    "nil\n",
    "true\n",
    "Waypoint\n",
    "\"summit\"\n",
    "12\n",
    "#<Encoding:UTF-8>\n"
);

#[test]
fn test_stdlib_marshal_round_trip_execution() {
    let output = run_example("stdlib/marshal_round_trip.rb");
    assert_eq!(output, MARSHAL_ROUND_TRIP_OUTPUT);
}

#[test]
fn test_stdlib_marshal_round_trip_parens_execution() {
    let output = run_example("stdlib/marshal_round_trip_parens.rb");
    assert_eq!(output, MARSHAL_ROUND_TRIP_OUTPUT);
}

/// The expected output of both `stdlib/erb_trim_modes` variants, which differ
/// only in whether the calls are written with parentheses.
const ERB_TRIM_MODES_OUTPUT: &str = concat!(
    "\"<ul>\\n\\n<li>1</li>\\n\\n<li>2</li>\\n\\n</ul>\\n\"\n",
    "\"<ul>\\n<li>1</li>\\n<li>2</li>\\n</ul>\\n\"\n",
    "\"<ul>\\n<li>1</li>\\n<li>2</li>\\n</ul>\\n\"\n",
    "\"<ul>\\n<li>1</li>\\n<li>2</li>\\n</ul>\\n\"\n",
    "\"<ul>\\n<li>1</li>\\n<li>2</li>\\n</ul>\\n%done\\n\"\n"
);

#[test]
fn test_stdlib_erb_trim_modes_execution() {
    let output = run_example("stdlib/erb_trim_modes.rb");
    assert_eq!(output, ERB_TRIM_MODES_OUTPUT);
}

#[test]
fn test_stdlib_erb_trim_modes_parens_execution() {
    let output = run_example("stdlib/erb_trim_modes_parens.rb");
    assert_eq!(output, ERB_TRIM_MODES_OUTPUT);
}

/// The expected output of both `stdlib/string_streams` variants, which differ
/// only in whether the calls are written with parentheses.
const STRING_STREAMS_OUTPUT: &str = "false\ntrue\ntrue\nfalse\n\"\"\nErrno::EACCES\n\"second\"\n";

#[test]
fn test_stdlib_string_streams_execution() {
    let output = run_example("stdlib/string_streams.rb");
    assert_eq!(output, STRING_STREAMS_OUTPUT);
}

#[test]
fn test_stdlib_string_streams_parens_execution() {
    let output = run_example("stdlib/string_streams_parens.rb");
    assert_eq!(output, STRING_STREAMS_OUTPUT);
}

const ZLIB_DEFLATE_OUTPUT: &str = concat!(
    "120,156,99,96,128,1,0,0,10,0,1\n",
    "120,156,51,52,132,1,0,10,145,1,235\n",
    "52\n",
    "120,187,20,225,3,203,75,76,74,78,73,77,75,207,200,204,2,0,21,134,3,248\n",
    "12 -> 13\n",
    "true\n",
    "2250 -> 72\n",
    "true\n",
    "157,5,0,36,10,0,0,0\n",
    "true\n",
);

#[test]
fn test_stdlib_zlib_deflate_execution() {
    let output = run_example("stdlib/zlib_deflate.rb");
    assert_eq!(output, ZLIB_DEFLATE_OUTPUT);
}

#[test]
fn test_stdlib_zlib_deflate_no_parens_execution() {
    let output = run_example("stdlib/zlib_deflate_no_parens.rb");
    assert_eq!(output, ZLIB_DEFLATE_OUTPUT);
}

const JSON_DOCUMENTS_OUTPUT: &str = concat!(
    "Ada\n[1815, 1852]\nnil\n2.5\n{a: 1}\n",
    "{\"a\":1,\"b\":[true,false,null]}\n",
    "{\n  \"a\": 1,\n  \"b\": [\n    1,\n    2\n  ]\n}\n",
    "{\"k\":\"v\"}\n[1,\"two\",null]\n\"a\\nbA\"\n[]\n{}\n",
    "refused: JSON::ParserError\n1.1111111111111112\n",
);

#[test]
fn test_stdlib_json_documents_execution() {
    let output = run_example("stdlib/json_documents.rb");
    assert_eq!(output, JSON_DOCUMENTS_OUTPUT);
}

#[test]
fn test_stdlib_json_documents_no_parens_execution() {
    let output = run_example("stdlib/json_documents_no_parens.rb");
    assert_eq!(output, JSON_DOCUMENTS_OUTPUT);
}

const OBJSPACE_DUMP_OUTPUT: &str = concat!(
    "STRING\nabc\n3\nUTF-8\nARRAY\nHASH\nString\ntrue\n",
    "wrong output option: #<Object:0x>\n",
);

#[test]
fn test_stdlib_objspace_dump_execution() {
    let output = run_example("stdlib/objspace_dump.rb");
    assert_eq!(output, OBJSPACE_DUMP_OUTPUT);
}

#[test]
fn test_stdlib_objspace_dump_no_parens_execution() {
    let output = run_example("stdlib/objspace_dump_no_parens.rb");
    assert_eq!(output, OBJSPACE_DUMP_OUTPUT);
}

/// The expected output of both `runtime/popen_options` variants.
const POPEN_OPTIONS_OUTPUT: &str = concat!(
    "\"hello\\n\"\n",
    "\"hi\\n\"\n",
    "\"to_err\\n\"\n",
    "\"/\\n\"\n",
    "\"named\\n\"\n",
    "true\n",
    "\"positional\\n\"\n",
    "\"bar\"\n",
    "Stream\n",
    "#<Encoding:EUC-JP>\n",
    "IOError\n",
    "\"foo\\n\"\n",
    "true\n",
    "\"logged\"\n",
    "\"No such file or directory -- does_not_exist (LoadError)\\n\"\n"
);

#[test]
fn test_runtime_popen_options_execution() {
    let output = run_example("runtime/popen_options.rb");
    assert_eq!(output, POPEN_OPTIONS_OUTPUT);
}

#[test]
fn test_runtime_popen_options_parens_execution() {
    let output = run_example("runtime/popen_options_parens.rb");
    assert_eq!(output, POPEN_OPTIONS_OUTPUT);
}

/// The expected output of both `runtime/process_title` variants.
const PROCESS_TITLE_OUTPUT: &str = concat!("\"metorex-title-example\"\n", "false\n");

#[test]
fn test_runtime_process_title_execution() {
    let output = run_example("runtime/process_title.rb");
    assert_eq!(output, PROCESS_TITLE_OUTPUT);
}

#[test]
fn test_runtime_process_title_no_parens_execution() {
    let output = run_example("runtime/process_title_no_parens.rb");
    assert_eq!(output, PROCESS_TITLE_OUTPUT);
}

/// The expected output of both `runtime/status_wait` variants.
const STATUS_WAIT_OUTPUT: &str = concat!("-1\n", "true\n", "Process::Status\n", "true\n");

#[test]
fn test_runtime_status_wait_execution() {
    let output = run_example("runtime/status_wait.rb");
    assert_eq!(output, STATUS_WAIT_OUTPUT);
}

#[test]
fn test_runtime_status_wait_no_parens_execution() {
    let output = run_example("runtime/status_wait_no_parens.rb");
    assert_eq!(output, STATUS_WAIT_OUTPUT);
}

#[test]
fn test_runtime_signal_zero_execution() {
    let output = run_example("runtime/signal_zero.rb");
    assert_eq!(output, "1\n");
}

#[test]
fn test_runtime_signal_zero_no_parens_execution() {
    let output = run_example("runtime/signal_zero_no_parens.rb");
    assert_eq!(output, "1\n");
}

/// The expected output of both `runtime/exec_with_options` variants.
const EXEC_WITH_OPTIONS_OUTPUT: &str = concat!(
    "Errno::ENOENT\n",
    "Errno::EACCES\n",
    "\"string contains null byte\"\n",
    "\"wrong first argument\"\n",
    "named_shell from the environment /\n",
);

#[test]
fn test_runtime_exec_with_options_execution() {
    let output = run_example("runtime/exec_with_options.rb");
    assert_eq!(output, EXEC_WITH_OPTIONS_OUTPUT);
}

#[test]
fn test_runtime_exec_with_options_no_parens_execution() {
    let output = run_example("runtime/exec_with_options_no_parens.rb");
    assert_eq!(output, EXEC_WITH_OPTIONS_OUTPUT);
}

#[test]
fn test_runtime_writing_to_a_closed_stderr_execution() {
    let output = run_example("runtime/writing_to_a_closed_stderr.rb");
    assert_eq!(output, "\"rescued Errno::EBADF\\n\"\n");
}

#[test]
fn test_runtime_writing_to_a_closed_stderr_no_parens_execution() {
    let output = run_example("runtime/writing_to_a_closed_stderr_no_parens.rb");
    assert_eq!(output, "\"rescued Errno::EBADF\\n\"\n");
}

/// The expected output of both `runtime/spawn_with_options` variants.
const SPAWN_WITH_OPTIONS_OUTPUT: &str = concat!(
    "\"hello\\ngone\\n\"\n",
    "[ArgumentError, \"negative process group ID : -1\"]\n",
    "[TypeError, \"no implicit conversion of Symbol into Integer\"]\n",
    "[ArgumentError, \"wrong exec option symbol: nonesuch\"]\n",
    "[ArgumentError, \"wrong exec option\"]\n",
    "[ArgumentError, \"environment name contains a equal : A=B\"]\n",
    "[TypeError, \"no implicit conversion of Symbol into String\"]\n",
    "[ArgumentError, \"wrong number of arguments (given 0, expected 1+)\"]\n",
);

#[test]
fn test_runtime_spawn_with_options_execution() {
    let output = run_example("runtime/spawn_with_options.rb");
    assert_eq!(output, SPAWN_WITH_OPTIONS_OUTPUT);
}

#[test]
fn test_runtime_spawn_with_options_no_parens_execution() {
    let output = run_example("runtime/spawn_with_options_no_parens.rb");
    assert_eq!(output, SPAWN_WITH_OPTIONS_OUTPUT);
}

/// The expected output of both `runtime/daemonizing` variants.
const DAEMONIZING_OUTPUT: &str = concat!("\"before\\n\"\n", "\"[true, true, \\\"/\\\", true]\"\n",);

#[test]
fn test_runtime_daemonizing_execution() {
    let output = run_example("runtime/daemonizing.rb");
    assert_eq!(output, DAEMONIZING_OUTPUT);
}

#[test]
fn test_runtime_daemonizing_no_parens_execution() {
    let output = run_example("runtime/daemonizing_no_parens.rb");
    assert_eq!(output, DAEMONIZING_OUTPUT);
}

/// The expected output of both `stdlib/marshal_wrapped_values` variants.
const MARSHAL_WRAPPED_VALUES_OUTPUT: &str = concat!(
    "\"\\x04\\bI\\\"\\ttext\\x06:\\x06ET\"\n",
    "\"\\x04\\b\\\"\\ttext\"\n",
    "\"\\x04\\bI:\\b\\xE2\\x86\\x92\\x06:\\x06ET\"\n",
    "\"\\x04\\bI/\\aa.\\x01\\x06:\\x06EF\"\n",
    "\"\\x04\\be:\\vTagged[\\x00\"\n",
    "\"\\x04\\bC:\\nWords[\\x00\"\n",
    "\"\\x04\\b}\\x00i\\x00\"\n",
    "\"\\x04\\bC:\\tHash{\\x00\"\n",
    "\"\\x04\\bC:\\nTable{\\x00\"\n",
    "\"\\x04\\bI\\\"\\tnote\\a:\\x06ET:\\b@byI\\\"\\ame\\x06;\\x00T\"\n",
    "\"can't dump hash with default proc\"\n",
);

#[test]
fn test_stdlib_marshal_wrapped_values_execution() {
    let output = run_example("stdlib/marshal_wrapped_values.rb");
    assert_eq!(output, MARSHAL_WRAPPED_VALUES_OUTPUT);
}

#[test]
fn test_stdlib_marshal_wrapped_values_no_parens_execution() {
    let output = run_example("stdlib/marshal_wrapped_values_no_parens.rb");
    assert_eq!(output, MARSHAL_WRAPPED_VALUES_OUTPUT);
}

/// The expected output of both `stdlib/marshal_objects` variants.
const MARSHAL_OBJECTS_OUTPUT: &str = concat!(
    "\"\\x04\\bo:\\fAccount\\x06:\\v@owner\\\"\\bann\"\n",
    "\"\\x04\\be:\\fLabeledo:\\fAccount\\x06:\\v@owner\\\"\\bann\"\n",
    "\"\\x04\\bS:\\nPoint\\a:\\x06xi\\x06:\\x06yi\\a\"\n",
    "\"\\x04\\bS:\\tSize\\a:\\nwidthi\\b:\\vheighti\\t\"\n",
    "\"\\x04\\bc\\fAccount\"\n",
    "\"\\x04\\bm\\fLabeled\"\n",
    "\"\\x04\\bU:\\rSnapshot[\\ai\\x06i\\a\"\n",
    "\"\\x04\\bu:\\vPacked\\vpacked\"\n",
    "[TypeError, \"can't dump anonymous class #<Class:0x...>\"]\n",
    "[TypeError, \"singleton class can't be dumped\"]\n",
    "[TypeError, \"no _dump_data is defined for class Proc\"]\n",
    "[ArgumentError, \"exceed depth limit\"]\n",
    "[TypeError, \"instance of IO needed\"]\n",
    "[:binmode, \"\"]\n",
);

#[test]
fn test_stdlib_marshal_objects_execution() {
    let output = run_example("stdlib/marshal_objects.rb");
    assert_eq!(output, MARSHAL_OBJECTS_OUTPUT);
}

#[test]
fn test_stdlib_marshal_objects_no_parens_execution() {
    let output = run_example("stdlib/marshal_objects_no_parens.rb");
    assert_eq!(output, MARSHAL_OBJECTS_OUTPUT);
}

/// The expected output of both `stdlib/marshal_core_values` variants.
const MARSHAL_CORE_VALUES_OUTPUT: &str = concat!(
    "\"\\x04\\bo:\\x11RuntimeError\\b:\\tmesgI\\\"\\x0Edisk full\\x06:\\x06ET:\\abt[\\x06I\\\"\\x10store.rb:12\\x06;\\aT:\\r@retriesi\\b\"\n",
    "[RuntimeError, \"write failed\", ArgumentError, \"bad size\"]\n",
    "\"\\x04\\bo:\\nRange\\b:\\texclF:\\nbegini\\x06:\\bendi\\a\"\n",
    "\"\\x04\\bo:\\nRange\\b:\\texclT:\\nbegini\\x06:\\bendi\\a\"\n",
    "3...9\n",
    "\"\\x04\\bIu:\\tTime\\r \\x00\\x1C\\xC0\\x00\\x00\\x00\\x00\\x06:\\tzoneI\\\"\\bUTC\\x06:\\x06EF\"\n",
    "true\n",
    "\"\\x04\\b[\\al+\\n\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x01\\x00@\\x06\"\n",
    "\"Exception\"\n",
);

#[test]
fn test_stdlib_marshal_core_values_execution() {
    let output = run_example("stdlib/marshal_core_values.rb");
    assert_eq!(output, MARSHAL_CORE_VALUES_OUTPUT);
}

#[test]
fn test_stdlib_marshal_core_values_no_parens_execution() {
    let output = run_example("stdlib/marshal_core_values_no_parens.rb");
    assert_eq!(output, MARSHAL_CORE_VALUES_OUTPUT);
}

/// The expected output of both `stdlib/marshal_load_handler` variants.
const MARSHAL_LOAD_HANDLER_OUTPUT: &str = concat!(
    "[\"foo\"]\n",
    "FOO\n",
    "false\n",
    "true\n",
    "true\n",
    "42\n",
    "false\n",
    "\"\\x04\\bIu:\\tTime\\r \\x80\\x11\\xC0@\\xE2\\x01\\x00\\t:\\rnano_numi\\x02\\x15\\x03:\\rnano_deni\\x06:\\rsubmicro\\\"\\ax\\x90:\\tzoneI\\\"\\bUTC\\x06:\\x06EF\"\n",
    "123456789\n",
    "true\n",
    "(1/3000000000)\n",
);

#[test]
fn test_stdlib_marshal_load_handler_execution() {
    let output = run_example("stdlib/marshal_load_handler.rb");
    assert_eq!(output, MARSHAL_LOAD_HANDLER_OUTPUT);
}

#[test]
fn test_stdlib_marshal_load_handler_no_parens_execution() {
    let output = run_example("stdlib/marshal_load_handler_no_parens.rb");
    assert_eq!(output, MARSHAL_LOAD_HANDLER_OUTPUT);
}

/// The expected output of both `stdlib/weakref_lifetime` variants.
const WEAKREF_LIFETIME_OUTPUT: &str =
    "true\ntrue\nfalse\n\"Invalid Reference - probably recycled\"\ntrue\n";

#[test]
fn test_stdlib_weakref_lifetime_execution() {
    let output = run_example("stdlib/weakref_lifetime.rb");
    assert_eq!(output, WEAKREF_LIFETIME_OUTPUT);
}

#[test]
fn test_stdlib_weakref_lifetime_no_parens_execution() {
    let output = run_example("stdlib/weakref_lifetime_no_parens.rb");
    assert_eq!(output, WEAKREF_LIFETIME_OUTPUT);
}

/// The expected output of both `stdlib/rubygems_interaction` variants.
const RUBYGEMS_INTERACTION_OUTPUT: &str = "\".]2;title. and . text\"\n[[:say, \"plain\"], [:say, \".[31mshown\"]]\n[[:say, \".]2;ok.\"], [:say, \"push: gone.\"], [:exit, 1]]\n[[:error, \"Invalid option: --.]2;x.. See 'gem --help'.\"], [:exit, 1], [:error, \"While executing gem ... (RuntimeError)\\n    bad.\"], [:exit, 1]]\nnil\n";

#[test]
fn test_stdlib_rubygems_interaction_execution() {
    let output = run_example("stdlib/rubygems_interaction.rb");
    assert_eq!(output, RUBYGEMS_INTERACTION_OUTPUT);
}

#[test]
fn test_stdlib_rubygems_interaction_no_parens_execution() {
    let output = run_example("stdlib/rubygems_interaction_no_parens.rb");
    assert_eq!(output, RUBYGEMS_INTERACTION_OUTPUT);
}

/// The expected output of both `stdlib/rubygems_owner` variants.
const RUBYGEMS_OWNER_OUTPUT: &str = "[\"owner\", \"Manage gem owners of a gem on the push server\", {}]\n[\"Owners for gem: rake\", \"- .]2;title. ()\", \"- maintainer ()\", \"- 7 ()\"]\n[\"runtime\"]\n";

#[test]
fn test_stdlib_rubygems_owner_execution() {
    let output = run_example("stdlib/rubygems_owner.rb");
    assert_eq!(output, RUBYGEMS_OWNER_OUTPUT);
}

#[test]
fn test_stdlib_rubygems_owner_no_parens_execution() {
    let output = run_example("stdlib/rubygems_owner_no_parens.rb");
    assert_eq!(output, RUBYGEMS_OWNER_OUTPUT);
}

/// The expected output of both `stdlib/mkmf_module` variants.
const MKMF_MODULE_OUTPUT: &str = "MakeMakefile\ntrue\n";

#[test]
fn test_stdlib_mkmf_module_execution() {
    let output = run_example("stdlib/mkmf_module.rb");
    assert_eq!(output, MKMF_MODULE_OUTPUT);
}

#[test]
fn test_stdlib_mkmf_module_no_parens_execution() {
    let output = run_example("stdlib/mkmf_module_no_parens.rb");
    assert_eq!(output, MKMF_MODULE_OUTPUT);
}

/// The expected output of both `stdlib/prime_generators` variants.
const PRIME_GENERATORS_OUTPUT: &str = "[2, 3, 5, 7, 11, 13, 17, 19]\n19\n[2, 3, 5, 7, 11, 13]\n[2, 3, 5]\n2\n2\n[2, 3, 5, 7, 11, 13, 17, 19, 23, 29]\n[2, 3, 5, 7, 11, 13, 17, 19]\n[[2, 1], [3, 2], [5, 3]]\ntrue\nfalse\n[[2, 3], [3, 2], [5, 1]]\n360\ntrue\n[true, true, false]\n[1, 2]\n[2, 3, 4, 5]\n[20, 30]\n";

#[test]
fn test_stdlib_prime_generators_execution() {
    let output = run_example("stdlib/prime_generators.rb");
    assert_eq!(output, PRIME_GENERATORS_OUTPUT);
}

#[test]
fn test_stdlib_prime_generators_no_parens_execution() {
    let output = run_example("stdlib/prime_generators_no_parens.rb");
    assert_eq!(output, PRIME_GENERATORS_OUTPUT);
}

/// The expected output of both `stdlib/bigdecimal_arithmetic` variants.
const BIGDECIMAL_ARITHMETIC_OUTPUT: &str = "0.33333333333333333333333333333333e0\n0.162000001474200013415220122078503110914378309320842614819667794858976933e46\n56\n3\n[4, Integer, 0.6e1]\n[-1, Infinity]\nFloatDomainError\n0.1234567e94\n0.1e1\n\"invalid value for BigDecimal(): \\\"1__2\\\"\"\nnil\n0.1e0\n0.33333e0\n-1\n0.15e1\n[\"loud\", \"0.4444e2\"]\n0.693147180559945309417232121458e0\n0.271828182845904523536028747135e1\n0.22314354220170971436e0\n[1.5, 0.75]\n[(10/1), (3/4)]\n0.25\n";

#[test]
fn test_stdlib_bigdecimal_arithmetic_execution() {
    let output = run_example("stdlib/bigdecimal_arithmetic.rb");
    assert_eq!(output, BIGDECIMAL_ARITHMETIC_OUTPUT);
}

#[test]
fn test_stdlib_bigdecimal_arithmetic_no_parens_execution() {
    let output = run_example("stdlib/bigdecimal_arithmetic_no_parens.rb");
    assert_eq!(output, BIGDECIMAL_ARITHMETIC_OUTPUT);
}

/// The expected output of both `stdlib/zlib_streaming` variants.
const ZLIB_STREAMING_OUTPUT: &str = "[16396, 16392, 7233]\n[16384, 16384, 7253]\n[nil, [16384, 16384, 7253], true]\n[[16384], 3632, true]\n[43759, 52790]\n";

#[test]
fn test_stdlib_zlib_streaming_execution() {
    let output = run_example("stdlib/zlib_streaming.rb");
    assert_eq!(output, ZLIB_STREAMING_OUTPUT);
}

#[test]
fn test_stdlib_zlib_streaming_no_parens_execution() {
    let output = run_example("stdlib/zlib_streaming_no_parens.rb");
    assert_eq!(output, ZLIB_STREAMING_OUTPUT);
}

/// The expected output of both `stdlib/matrix_eigen` variants.
const MATRIX_EIGEN_OUTPUT: &str = "[1.8548973088, 3.4760236029, 6.6690790883]\n3\ntrue\n[(0.0+1.0i), (0.0-1.0i)]\n[[(-1.0+0.0i), (0.0+1.0i)], [(-1.0-0.0i), (0.0-1.0i)]]\n5\nExceptionForMatrix::ErrDimensionMismatch\n\"Expected Matrix but got Integer\"\n";

#[test]
fn test_stdlib_matrix_eigen_execution() {
    let output = run_example("stdlib/matrix_eigen.rb");
    assert_eq!(output, MATRIX_EIGEN_OUTPUT);
}

#[test]
fn test_stdlib_matrix_eigen_no_parens_execution() {
    let output = run_example("stdlib/matrix_eigen_no_parens.rb");
    assert_eq!(output, MATRIX_EIGEN_OUTPUT);
}

/// The expected output of both `stdlib/allocation_tracing` variants.
const ALLOCATION_TRACING_OUTPUT: &str = "[true, 14, \"Shelf\", :stock]\n[true, 14, \"Shelf\", :stock]\n[true, 14, \"Shelf\", :stock]\n[true, 14, \"Shelf\", :stock]\n[true, 18, nil, :build]\n[true, 24, \"Labeled\", :label]\n[true, 36, nil, nil]\ntrue\n[false, nil, nil, nil]\n[false, nil, nil, nil]\n[false, nil, nil, nil]\n[false, nil, nil, nil]\n:finished\n50\nnil\n";

#[test]
fn test_stdlib_allocation_tracing_execution() {
    let output = run_example("stdlib/allocation_tracing.rb");
    assert_eq!(output, ALLOCATION_TRACING_OUTPUT);
}

#[test]
fn test_stdlib_allocation_tracing_no_parens_execution() {
    let output = run_example("stdlib/allocation_tracing_no_parens.rb");
    assert_eq!(output, ALLOCATION_TRACING_OUTPUT);
}

const CONSOLE_PP_AND_PATHNAME_OUTPUT: &str = concat!(
    "[30, 100]\n",
    "true\n",
    "false\n",
    "false\n",
    "false\n",
    "false\n",
    "true\n",
    "\"q\"\n",
    "\"\\e[3;5H\\e[0K\\e[1A\"\n",
    "#<ArgumentError: wrong screen erase mode: 9>\n",
    "true\n",
    "true\n",
    "nil\n",
    "nil\n",
    "\"Password: \\n\"\n",
    "{id: 1042,\n",
    " lines:\n",
    "  [{sku: \"SKU-1\", quantity: 1},\n",
    "   {sku: \"SKU-2\", quantity: 2},\n",
    "   {sku: \"SKU-3\", quantity: 3},\n",
    "   {sku: \"SKU-4\", quantity: 4},\n",
    "   {sku: \"SKU-5\", quantity: 5},\n",
    "   {sku: \"SKU-6\", quantity: 6},\n",
    "   {sku: \"SKU-7\", quantity: 7},\n",
    "   {sku: \"SKU-8\", quantity: 8},\n",
    "   {sku: \"SKU-9\", quantity: 9},\n",
    "   {sku: \"SKU-10\", quantity: 10},\n",
    "   {sku: \"SKU-11\", quantity: 11},\n",
    "   {sku: \"SKU-12\", quantity: 12}]}\n",
    "14\n",
    "\"constant\"\n",
    "#<Pathname:app.log>\n",
    "\"/var/log/app.log\"\n",
    "#<Pathname:app.log>\n",
    "-1\n",
    "\"<internal:pathname_builtin>\"\n",
    "true\n",
    "true\n",
);

#[test]
fn test_stdlib_console_pp_and_pathname_execution() {
    let output = run_example("stdlib/console_pp_and_pathname.rb");
    assert_eq!(output, CONSOLE_PP_AND_PATHNAME_OUTPUT);
}

#[test]
fn test_stdlib_console_pp_and_pathname_no_parens_execution() {
    let output = run_example("stdlib/console_pp_and_pathname_no_parens.rb");
    assert_eq!(output, CONSOLE_PP_AND_PATHNAME_OUTPUT);
}

/// The expected output of both `runtime/pattern_matching` variants.
const PATTERN_MATCHING_OUTPUT: &str = concat!(
    "The answer!\n",
    "Matched: 100\n",
    "a=1, b=2, c=3\n",
    "First: 1\n",
    "Rest: [2, 3, 4, 5]\n",
    "First: 1, Last: 5\n",
    "Middle: [2, 3, 4]\n",
    "Point at (10, 20)\n",
    "Warm\n",
    "Sum: 10\n",
    "Number: 1\n",
    "String: hello\n",
    "Boolean: true\n",
    "Not Found\n",
    "Starting...\n",
    "Flag is true\n",
    "No value\n",
    "Working age\n",
    "First is 1, last is 4\n",
);

#[test]
fn test_runtime_pattern_matching_execution() {
    let output = run_example("runtime/pattern_matching.rb");
    assert_eq!(output, PATTERN_MATCHING_OUTPUT);
}

#[test]
fn test_runtime_pattern_matching_parens_execution() {
    let output = run_example("runtime/pattern_matching_parens.rb");
    assert_eq!(output, PATTERN_MATCHING_OUTPUT);
}

/// The expected output of both `runtime/error_reporting` variants.
const ERROR_REPORTING_OUTPUT: &str = concat!(
    "Caught error: divided by 0\n",
    "Type error caught: no implicit conversion of Integer into String\n",
    "Method error: undefined method 'undefined_method' for an instance of Integer\n",
    "Error with stack trace: Cannot divide by zero!\n",
    "Pattern match error: 5\n",
    "Control flow error: break from proc-closure\n",
    "Entering level 1\n",
    "Entering level 2\n",
    "Entering level 3\n",
    "Entering level 4\n",
    "Entering level 5\n",
    "Deep error caught: undefined local variable or method 'undefined_variable' for an instance of Deep\n",
    "Argument error: wrong number of arguments (given 1, expected 2)\n",
    "Assignment error: SyntaxError\n",
    "Callable error: undefined method 'x' for main\n",
    "Guard error: undefined local variable or method 'undefined_var' for main\n",
    "Custom exception caught\n",
    "Error reporting examples completed!\n",
);

#[test]
fn test_runtime_error_reporting_execution() {
    let output = run_example("runtime/error_reporting.rb");
    assert_eq!(output, ERROR_REPORTING_OUTPUT);
}

#[test]
fn test_runtime_error_reporting_parens_execution() {
    let output = run_example("runtime/error_reporting_parens.rb");
    assert_eq!(output, ERROR_REPORTING_OUTPUT);
}

/// The expected output of both `stdlib/ipaddr_before_socket` variants.
const IPADDR_BEFORE_SOCKET_OUTPUT: &str = "Class\ntrue\ntrue\ntrue\n\"127.0.0.1\"\n";

#[test]
fn test_stdlib_ipaddr_before_socket_execution() {
    let output = run_example("stdlib/ipaddr_before_socket.rb");
    assert_eq!(output, IPADDR_BEFORE_SOCKET_OUTPUT);
}

#[test]
fn test_stdlib_ipaddr_before_socket_no_parens_execution() {
    let output = run_example("stdlib/ipaddr_before_socket_no_parens.rb");
    assert_eq!(output, IPADDR_BEFORE_SOCKET_OUTPUT);
}

/// The expected output of both `stdlib/timeout_busy_code` variants.
const TIMEOUT_BUSY_CODE_OUTPUT: &str = concat!(
    "[Timeout::Error, \"execution expired\"]\n",
    "[Timeout::Error, \"execution expired\"]\n",
    "[ArgumentError, \"slow\"]\n",
    ":ensured\n",
    ":raised_once\n",
    ":inner_limit\n",
    "42\n",
);

#[test]
fn test_stdlib_timeout_busy_code_execution() {
    let output = run_example("stdlib/timeout_busy_code.rb");
    assert_eq!(output, TIMEOUT_BUSY_CODE_OUTPUT);
}

#[test]
fn test_stdlib_timeout_busy_code_no_parens_execution() {
    let output = run_example("stdlib/timeout_busy_code_no_parens.rb");
    assert_eq!(output, TIMEOUT_BUSY_CODE_OUTPUT);
}

/// The expected output of `runtime/builtin_classes`.
const BUILTIN_CLASSES: &str = concat!(
    "=== Object Class ===\n",
    "x.class: Integer\n",
    "x.to_s: 42\n",
    "x.respond_to?('to_s'): true\n",
    "y.class: String\n",
    "y.to_s: hello\n",
    "\n",
    "=== String Class ===\n",
    "Original: Hello, World!\n",
    "Length: 13\n",
    "Uppercase: HELLO, WORLD!\n",
    "Lowercase: hello, world!\n",
    "Concatenated: Hello, Alice!\n",
    "\n",
    "=== Integer Class ===\n",
    "num1: 42\n",
    "num2: 10\n",
    "num1.class: Integer\n",
    "\n",
    "=== Float Class ===\n",
    "pi: 3.14159\n",
    "e: 2.71828\n",
    "pi.class: Float\n",
    "\n",
    "=== Array Class ===\n",
    "Array: [1, 2, 3, 4, 5]\n",
    "Length: 5\n",
    "After push(6): [1, 2, 3, 4, 5, 6]\n",
    "Popped: 6\n",
    "After pop: [1, 2, 3, 4, 5]\n",
    "First element: 1\n",
    "Third element: 3\n",
    "Mixed array: [1, \"two\", 3.0, true, nil]\n",
    "\n",
    "=== Hash Class ===\n",
    "Hash: {name: \"Bob\", age: 30, city: \"New York\"}\n",
    "Name: Bob\n",
    "Age: 30\n",
    "After adding job: {name: \"Bob\", age: 30, city: \"New York\", job: \"Engineer\"}\n",
    "\n",
    "=== Set Class ===\n",
    "Set: Set[1, 2, 3]\n",
    "Contains 2: true\n",
    "Contains 5: false\n",
    "\n",
    "=== Exception Hierarchy ===\n",
    "Caught Exception: Generic exception\n",
    "Caught StandardError: Standard error occurred\n",
    "Caught RuntimeError: Runtime error occurred\n",
    "Caught TypeError: Expected Integer, got String\n",
    "Caught ArgumentError: Invalid value provided\n",
    "\n",
    "=== Exception Catching ===\n",
    "Value error: Division by zero\n",
    "Caught via StandardError: Runtime problem\n",
    "\n",
    "=== Class Hierarchy ===\n",
    "Integer is a subclass of Object: true\n",
    "String is a subclass of Object: true\n",
    "RuntimeError is a subclass of StandardError: true\n",
    "StandardError is a subclass of Exception: true\n",
    "Exception is a subclass of Object: true\n",
    "\n",
    "=== Inherited Methods ===\n",
    "Array has to_s: true\n",
    "String has to_s: true\n",
    "Integer has to_s: true\n",
    "\n",
    "=== Custom Classes ===\n",
    "Person instance: Person(name: Charlie, age: 25)\n",
    "Person class: Person\n",
    "Person has to_s: true\n",
    "Person is a subclass of Object: true\n",
    "\n",
    "=== Polymorphism ===\n",
    "Unknown type: Integer\n",
    "Unknown type: Float\n",
    "Unknown type: String\n",
    "Unknown type: Array\n",
    "\n",
    "=== Type Checking ===\n",
    "Is Integer: true\n",
    "Is Float: false\n",
    "Is Object: true\n",
    "Is String: false\n",
    "Array is Array: true\n",
    "Array is Object: true\n",
);

#[test]
fn test_runtime_builtin_classes_execution() {
    let output = run_example("runtime/builtin_classes.rb");
    assert_eq!(output, BUILTIN_CLASSES);
}

#[test]
fn test_a_missing_program_file_is_reported_under_the_name_the_interpreter_was_started_by() {
    let started_by = env!("CARGO_BIN_EXE_metorex");
    let output = Command::new(started_by)
        .arg("does_not_exist")
        .output()
        .expect("failed to run metorex");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr was not utf8"),
        format!("{started_by}: No such file or directory -- does_not_exist (LoadError)\n")
    );
}

#[test]
fn test_an_invalid_rubyopt_switch_is_reported_under_the_name_the_interpreter_was_started_by() {
    let started_by = env!("CARGO_BIN_EXE_metorex");
    let output = Command::new(started_by)
        .env("RUBYOPT", "-e")
        .args(["-e", "1"])
        .output()
        .expect("failed to run metorex");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr was not utf8"),
        format!("{started_by}: invalid switch in RUBYOPT: -e (RuntimeError)\n")
    );
}

/// The expected output of both `runtime/release_named` variants.
const RELEASE_NAMED: &str = concat!("\"4.0.7\"\n", "0\n", "\"2026-09-15\"\n", "true\n");

#[test]
fn test_runtime_release_named_execution() {
    let output = run_example("runtime/release_named.rb");
    assert_eq!(output, RELEASE_NAMED);
}

#[test]
fn test_runtime_release_named_no_parens_execution() {
    let output = run_example("runtime/release_named_no_parens.rb");
    assert_eq!(output, RELEASE_NAMED);
}

/// The expected output of both `stdlib/rbconfig_cross_compiling` variants.
const RBCONFIG_CROSS_COMPILING_OUTPUT: &str = "nil\n\"no\"\ntrue\n";

#[test]
fn test_stdlib_rbconfig_cross_compiling_execution() {
    let output = run_example("stdlib/rbconfig_cross_compiling.rb");
    assert_eq!(output, RBCONFIG_CROSS_COMPILING_OUTPUT);
}

#[test]
fn test_stdlib_rbconfig_cross_compiling_no_parens_execution() {
    let output = run_example("stdlib/rbconfig_cross_compiling_no_parens.rb");
    assert_eq!(output, RBCONFIG_CROSS_COMPILING_OUTPUT);
}

/// The expected output of both `stdlib/marshal_extended_values` variants.
const MARSHAL_EXTENDED_VALUES_OUTPUT: &str = "true\ntrue\nfalse\ntrue\nfalse\n";

#[test]
fn test_stdlib_marshal_extended_values_execution() {
    let output = run_example("stdlib/marshal_extended_values.rb");
    assert_eq!(output, MARSHAL_EXTENDED_VALUES_OUTPUT);
}

#[test]
fn test_stdlib_marshal_extended_values_no_parens_execution() {
    let output = run_example("stdlib/marshal_extended_values_no_parens.rb");
    assert_eq!(output, MARSHAL_EXTENDED_VALUES_OUTPUT);
}

#[test]
fn test_io_clone_holds_a_descriptor_of_its_own() {
    let expected = "false\nstdout still open\ncaptured \"from a child\\n\"\n";
    let output = run_example("stdlib/io/io_clone_descriptor.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_line_coverage_counts_each_kind_of_conditional_as_mri_does() {
    let expected = "[1, 2, 1, 1, 1, nil, 0, nil, nil, nil, 1, 1, nil, nil, 1, 1, nil, 0, nil, 1, nil, 0, nil, nil, nil, 1, 1, 1, nil, nil, nil, 1, 1, 1, 2, 2, nil, 1, nil, nil, 1, 1, 1, nil, nil, 1, 1, 1, 1, 1, 1, 1]\n";
    let output = run_example("coverage/conditional_lines.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_branch_coverage_counts_each_arm_of_each_conditional() {
    let expected = "if at 2: then@3=1 elsif@5=1 else@7=0\nif at 12: then@12=1 else@12=0\ncase at 16: when@18=0 when@20=1 else@22=0\nunless at 27: then@28=1 else@27=0\nwhile at 34: body@35=2\n&. at 42: then@42=0 else@42=1\n";
    let output = run_example("coverage/branch_arms.rb");
    assert_eq!(output, expected);
}

#[test]
fn syntax_suggest_shows_the_lines_around_a_syntax_error() {
    let expected = "--> refused.rb\n\nunexpected integer, expecting end-of-input\n\n> 2  puts \"started\"\n> 3  puts 1 2\n";
    let output = run_example("stdlib/syntax_suggest_search.rb");
    assert_eq!(output, expected);
}

#[test]
fn marshal_dump_leaves_literal_strings_unchanged_and_prints_no_warning() {
    let output = run_example("stdlib/marshal_dump_warns_nothing.rb");
    assert_eq!(
        output,
        "#<struct Pair left=7, right=\"something\">\ntrue\nfalse\ntrue\n"
    );
}

#[test]
fn yaml_dump_of_a_struct_prints_no_warning() {
    let output = run_example("stdlib/yaml_dump_of_a_struct_warns_nothing.rb");
    assert_eq!(
        output,
        "--- !ruby/struct:Pair\nleft: 7\nright: something\n--- !ruby/struct:Pair\nleft: 7\nright: something\n"
    );
}

#[test]
fn a_drb_service_started_after_install_acl_answers_the_hosts_it_allows() {
    let output = run_example("stdlib/drb_service_with_an_acl.rb");
    assert_eq!(output, "true\n75\nfalse\n");
}

#[test]
fn stringio_writes_into_the_string_it_was_opened_over() {
    let output = run_example("stdlib/stringio_writes_into_its_string.rb");
    assert_eq!(
        output,
        "\"first\\nsecond\"\ntrue\n\"Xbc\"\n\"one\"\n\"after\"\n"
    );
}

#[test]
fn an_erb_template_error_names_the_template_line() {
    let output = run_example("stdlib/erb_error_line.rb");
    assert_eq!(output, "(erb):1:in '<main>'\n(erb):2:in '<main>'\n");
}

#[test]
fn a_drb_object_reaches_its_server_after_another_service_starts() {
    let output = run_example("stdlib/drb_call_after_a_second_service.rb");
    assert_eq!(output, "75\n");
}

#[test]
fn the_bundled_gems_load_with_no_gem_directory_holding_them() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let empty = std::env::temp_dir().join(format!("metorex_no_gems_{}", std::process::id()));
    std::fs::create_dir_all(&empty).expect("the temporary directory is writable");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_metorex"))
        .current_dir(manifest_dir)
        .arg(format!(
            "{}/stdlib/carried_bundled_gems.rb",
            crate::common::EXAMPLES_DIR
        ))
        .env("GEM_HOME", &empty)
        .env("GEM_PATH", &empty)
        .output()
        .expect("failed to execute example");
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        concat!(
            "true\n",
            "Test::Unit::Assertions\n",
            "true\n",
            "\"entries\"\n",
            "\"120\"\n",
            "\"ContentEncoded\"\n",
            "\"Entw&APw-rfe\"\n",
            "110\n",
            "25\n",
            "\"\\n<p>Due <strong>2026-11-01</strong></p>\\n\"\n",
            "\"Array[Integer]\"\n",
            "\"(3,4)\"\n",
            "true\n",
            "true\n",
            "true\n",
        )
    );
}

#[test]
fn a_file_requiring_test_unit_runs_its_tests_as_the_program_ends() {
    let output = run_example("stdlib/test_unit_autorun.rb");
    assert!(
        output.contains("1 tests, 1 assertions, 0 failures, 0 errors"),
        "{output}"
    );
}

#[test]
fn an_ssl_context_sets_up_once_and_a_key_reads_from_a_file() {
    let output = run_example("stdlib/openssl_context_setup_and_key_files.rb");
    assert_eq!(
        output,
        "true\ntrue\nnil\ntrue\n\"no implicit conversion of Object into String\"\n\"no implicit conversion of Float into String\"\n"
    );
}

#[test]
fn a_delegator_with_nothing_delegated_answers_respond_to() {
    let output = run_example("stdlib/simple_delegator_before_delegating.rb");
    assert_eq!(
        output,
        "false\ntrue\n:nothing_delegated\n\"not delegated\"\ntrue\n42\n"
    );
}

#[test]
fn an_allocated_open_struct_answers_respond_to() {
    let output = run_example("stdlib/allocated_open_struct.rb");
    assert_eq!(output, "false\n");
}

#[test]
fn yaml_reads_quoted_scalars_across_lines() {
    let output = run_example("stdlib/yaml_quoted_scalars_across_lines.rb");
    assert_eq!(
        output,
        "{\"first\" => \"RUBY0001 OK [Error=\\\"Exchange: Cross Server access\\\" Service=Imap4] done.\\r\\n\", \"second\" => \"[Error=\\\"Exchange: Cross access\\\" more\\nafter a blank line\", \"third\" => \"ends joined\", \"fourth\" => \"plain\"}\n"
    );
}

#[test]
fn yaml_dump_writes_each_value_in_the_style_psych_chooses() {
    let output = run_example("stdlib/yaml_dump_styles.rb");
    assert_eq!(
        output,
        "---\nplain: maintenance window\nquoted: '02:00'\nboolean_word: 'yes'\nempty: ''\nmultiline: |\n  first line\n  second line\ntrailing_space: 'ends with a space '\nunicode: café ☃\ncontrol: \"tab\\there\"\nsymbol: :ready\nnumbers:\n- 1\n- 2.5\n- -3\n- 100000000000000000000\nnothing:\nflags:\n- true\n- false\nnested:\n  empty_list: []\n  empty_map: {}\n  list:\n  - a\n  - b\nstruct: !ruby/struct:Reading\n  sensor: north\n  value: 21.5\nrange: !ruby/range\n  begin: 1\n  end: 3\n  excl: false\nlong: 'word word word word word word word word word word word word word word word\n  word word word word word word word word word word word word word word word '\n---\n- 1\n- - 2\n  - - 3\n---\na: 1\n--- two\n---\n- 3\n---\nfirst: &1\n- x\n- \"y\"\nsecond: *1\n{\"plain\" => \"maintenance window\", \"quoted\" => \"02:00\", \"boolean_word\" => \"yes\", \"empty\" => \"\", \"multiline\" => \"first line\\nsecond line\\n\", \"trailing_space\" => \"ends with a space \", \"unicode\" => \"café ☃\", \"control\" => \"tab\\there\", \"numbers\" => [1, 2.5, -3, 100000000000000000000], \"nothing\" => nil, \"flags\" => [true, false], \"nested\" => {\"empty_list\" => [], \"empty_map\" => {}, \"list\" => [\"a\", \"b\"]}, \"long\" => \"word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word \"}\n"
    );
}

#[test]
fn rbconfig_names_the_library_directories_under_the_prefix() {
    let output = run_example("stdlib/rbconfig_library_directories.rb");
    assert_eq!(output, "true\ntrue\ntrue\n");
}

#[test]
fn bigdecimal_is_made_by_its_kernel_function_and_has_no_new() {
    let output = run_example("stdlib/bigdecimal_has_no_new.rb");
    assert_eq!(
        output,
        "false\nundefined method 'new' for class BigDecimal\n0.25e1\n"
    );
}

#[test]
fn bigdecimal_converts_floats_and_rationals_through_its_c_extension() {
    let output = run_example("stdlib/bigdecimal_from_float_and_rational.rb");
    assert_eq!(output, "0.905e1\n0.905e1\n0.3333333333e0\n0.31429e1\n");
}

#[test]
fn json_generates_subclasses_fragments_and_coder_output_through_its_c_extension() {
    let output = run_example("stdlib/json_extension_objects.rb");
    assert_eq!(
        output,
        concat!(
            "\"[\\\"a\\\",\\\"b\\\"]\"\n",
            "\"\\\"name\\\"\"\n",
            "\"{\\\"theme\\\":\\\"dark\\\"}\"\n",
            "\"{\\\"retries\\\":3}\"\n",
            "\"{\\\"raw\\\":[1, 2]}\"\n",
            "\"\\\"converted\\\"\"\n",
            "\"[\\\"symbol\\\",\\\"1..2\\\"]\"\n",
            "248891\n",
            "true\n",
            "Float\n",
        )
    );
}

#[test]
fn rbconfig_limits_name_the_pointer_size_and_float_ranges() {
    let output = run_example("stdlib/rbconfig_pointer_and_float_limits.rb");
    assert_eq!(
        output,
        concat!(
            "INTPTR_MAX 9223372036854775807\nINTPTR_MIN -9223372036854775808\n",
            "UINTPTR_MAX 18446744073709551615\nSIZE_MAX 18446744073709551615\n",
            "PTRDIFF_MIN -9223372036854775808\nINTMAX_MAX 9223372036854775807\n",
            "UINT_LEAST8_MAX 255\nDBL_MANT_DIG 53\nDBL_EPSILON 2.220446049250313e-16\n",
            "DBL_TRUE_MIN 5.0e-324\nFLT_DIG 6\nFLT_RADIX 2\n",
        )
    );
}

#[test]
fn bigdecimal_calls_operators_its_c_code_names_by_character() {
    let output = run_example("stdlib/bigdecimal_operator_calls.rb");
    assert_eq!(output, "(3/2)\n(-9/4)\n[3, 0.1e1]\n10\ntrue\n");
}

#[test]
fn bigdecimal_orders_in_ranges_and_divides_inside_complex_numbers() {
    let output = run_example("stdlib/bigdecimal_in_ranges_and_complex.rb");
    assert_eq!(
        output,
        concat!(
            "0.5e0\ntrue\n\"can't do binary search for BigDecimal\"\n",
            "(0.0-0.3e1i)\n(0.6e1+0i)\n",
        )
    );
}

#[test]
fn unsigned_certificates_count_as_equal() {
    let output = run_example("stdlib/openssl_unsigned_certificates_equal.rb");
    assert_eq!(output, "true\ntrue\ntrue\n");
}

#[test]
fn cgi_escape_answers_in_the_encoding_it_was_given() {
    let output = run_example("stdlib/cgi_escape_keeps_the_encoding.rb");
    assert_eq!(
        output,
        concat!(
            "\"%E9\"\n#<Encoding:ISO-8859-1>\n",
            "\"%80\"\n#<Encoding:BINARY (ASCII-8BIT)>\n",
            "\"a+b~%C3%A9\"\n#<Encoding:UTF-8>\n",
        )
    );
}

#[test]
fn string_scanner_peek_answers_bytes() {
    let output = run_example("stdlib/strscan_peek_bytes.rb");
    assert_eq!(
        output,
        "\"a\\xC3\"\n#<Encoding:UTF-8>\n\"aé\"\n\"aé b\"\n\"\\xC3\"\n\" b\"\n1999999\n"
    );
}

#[test]
fn string_io_read_with_a_length_reads_an_ascii_run_in_one_piece() {
    let output = run_example("stdlib/stringio_read_ascii_run.rb");
    assert_eq!(
        output,
        "\"ab\"\n\"cd\"\n\"cd\"\n4\n\"ef\"\nnil\n\"\"\n2999999\n2999999\n\"\\x00\"\n"
    );
}

#[test]
fn a_gzip_header_carries_the_time_it_was_written() {
    let output = run_example("stdlib/gzip_header_time.rb");
    assert_eq!(output, "true\ntrue\n1700000000\n");
}

#[test]
fn mkmf_checks_link_against_the_interpreter_and_write_the_extconf_header() {
    let output = run_example("stdlib/mkmf_checks_and_extconf_header.rb");
    assert_eq!(
        output,
        concat!(
            "checking for rb_hash_new_capa() in ruby.h... yes\n",
            "true\n",
            "checking for rb_enc_interned_str() in ruby.h... yes\n",
            "true\n",
            "checking for rb_no_function_has_this_name()... no\n",
            "false\n",
            "checking for whether -Wall is accepted as CFLAGS... yes\n",
            "checking for whether -Wno-flag-the-compiler-knows is accepted as CFLAGS... no\n",
            "true\n",
            "false\n",
            "true\n",
            "false\n",
            "\"-Werror\"\n",
            "nil\n",
            "String\n",
            "creating extconf.h\n",
            "#ifndef EXTCONF_H\n",
            "#define EXTCONF_H\n",
            "#define HAVE_RB_HASH_NEW_CAPA 1\n",
            "#define HAVE_RB_ENC_INTERNED_STR 1\n",
            "#define THE_ANSWER 42\n",
            "#endif\n",
            "creating Makefile\n",
            "RUBY_EXTCONF_H = extconf.h\n",
            "true\n",
            "false\n",
            "true\n",
            "true\n",
            "true\n",
            "true\n",
        )
    );
}

#[test]
fn cgi_escape_writes_each_reserved_byte_in_hex_and_unescape_reads_it_back() {
    let output = run_example("stdlib/cgi_escape_reads_bytes.rb");
    assert_eq!(
        output,
        concat!(
            "[\"a+b%26c%3Dd%2Fe~f_g.h-i\", #<Encoding:UTF-8>]\n",
            "\"a%20b%26c%3Dd%2Fe~f_g.h-i\"\n",
            "true\n",
            "[\"caf%C3%A9+%E6%97%A5%E6%9C%AC\", #<Encoding:UTF-8>]\n",
            "\"caf%C3%A9%20%E6%97%A5%E6%9C%AC\"\n",
            "true\n",
            "[\"bad+%FF%FE+end\", #<Encoding:BINARY (ASCII-8BIT)>]\n",
            "\"bad%20%FF%FE%20end\"\n",
            "true\n",
            "[\"plus%2Bpercent%25\", #<Encoding:UTF-8>]\n",
            "\"plus%2Bpercent%25\"\n",
            "true\n",
            "[\"\", #<Encoding:UTF-8>]\n",
            "\"\"\n",
            "true\n",
            "120000\n",
        )
    );
}

#[test]
fn openssl_names_compare_by_canonical_form_and_a_copy_takes_its_own_entries() {
    let output = run_example("stdlib/openssl_name_copy_and_comparison.rb");
    assert_eq!(
        output,
        concat!(
            "true\n",
            "true\n",
            "\"/C=US/O=Example/CN=Example Root/OU=Unit\"\n",
            "\"/C=US/O=Example/CN=Example Root\"\n",
            "false\n",
            "true\n",
        )
    );
}

#[test]
fn an_extension_factory_builds_extended_key_usage_and_a_netscape_comment() {
    let output = run_example("stdlib/openssl_extension_factory_purposes_and_comment.rb");
    assert_eq!(
        output,
        concat!(
            "[\"extendedKeyUsage\", \"TLS Web Server Authentication, TLS Web Client Authentication\", false]\n",
            "\"301d0603551d250416301406082b0601050507030106082b06010505070302\"\n",
            "[\"nsComment\", \"Generated by Ruby/OpenSSL\", false]\n",
            "\"302806096086480186f842010d041b161947656e65726174656420627920527562792f4f70656e53534c\"\n",
            "OpenSSL::X509::ExtensionError\n",
        )
    );
}

#[test]
fn openssl_aes_in_counter_mode() {
    let output = run_example("stdlib/openssl_aes_counter_mode.rb");
    assert_eq!(
        output,
        concat!(
            "\"AES-128-CTR\"\n1\n16\n16\n",
            "\"874d6191b620e3261bef6864990db6ce9806f66b7970fdff8617187bb9fffdff\"\n",
            "[5, 20, 7]\ntrue\ntrue\n",
            "\"411c491b9337de26117d4f7c585e753cbfa02109e4ad2076d196c53fc911ea811cab71419fef79e2\"\n",
            "\"AES-128-CBC\"\n\"AES-192-CBC\"\n",
        )
    );
}

#[test]
fn zlib_raw_streams_inflated_in_chunks() {
    let output = run_example("stdlib/zlib_raw_inflate_in_chunks.rb");
    assert_eq!(
        output,
        concat!(
            "true\n",
            "true\n",
            "11\n",
            "true\n",
            "94893\n",
            "\"trailing bytes after the stream\"\n",
            "94893\n",
            "true\n",
            "300\n",
            "true\n",
            "9\n",
            "9\n",
            "\"\"\n",
            "\"TAILMORE\"\n",
            "\"\"\n",
            "4\n",
            "5\n",
            "false\n",
            "296\n",
            "9\n",
            "true\n",
            "300\n",
            "\"XY\"\n",
            "\"\"\n",
            "\"hello\"\n",
            "true\n",
            "13\n",
            "\"rest\"\n",
        )
    );
}

#[test]
fn openssl_ec_keys_and_ecdsa_signatures() {
    let output = run_example("stdlib/openssl_ec_keys_and_ecdsa.rb");
    assert_eq!(
        output,
        concat!(
            "OpenSSL::PKey::EC\n",
            "\"secp384r1\"\n",
            "384\n",
            "true\n",
            "true\n",
            "\"03ad0aef0370f6159fd5d17adc894007f09f8ca126b7b110246858dbafd3a093dbfe626af36ec3efc7c7a53214414c0c58\"\n",
            "true\n",
            "false\n",
            "true\n",
            "true\n",
            "false\n",
            "true\n",
            "true\n",
            "true\n",
            "true\n",
            "[\"prime256v1\", true, true]\n",
            "true\n",
            "true\n",
            "[\"secp256k1\", true, true]\n",
            "true\n",
            "true\n",
            "[\"secp521r1\", true, true]\n",
            "true\n",
            "true\n",
            "[\"prime239v2\", true, true]\n",
            "true\n",
            "true\n",
            "false\n",
            "false\n",
            "\"FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551\"\n",
            "OpenSSL::PKey::PKeyError\n",
        )
    );
}

#[test]
fn openssl_rsa_pss_signatures() {
    let output = run_example("stdlib/openssl_rsa_pss_signatures.rb");
    assert_eq!(
        output,
        concat!(
            "true\n",
            "true\n",
            "false\n",
            "\"no implicit conversion of Symbol into Integer\"\n",
            "true\n",
            "false\n",
            "128\n",
            "false\n",
            "true\n",
            "true\n",
            "\"private key is needed\"\n",
        )
    );
}

#[test]
fn openssl_certificate_revocation_lists() {
    let output = run_example("stdlib/openssl_certificate_revocation_lists.rb");
    assert_eq!(
        output,
        concat!(
            "[\"crlDistributionPoints = Full Name:,   URI:http://www.example.com/crl, , Full Name:,   URI:http://backup.example.com/crl, \"]\n",
            "1\n",
            "\"/DC=org/DC=fake-ca/CN=Fake CA\"\n",
            "\"sha256WithRSAEncryption\"\n",
            "[\"crlNumber = 1\"]\n",
            "true\n",
            "false\n",
            "\"-----BEGIN X509 CRL-----\\n\"\n",
            "{\"URI.1\" => \"http://www.example.com/crl\", \"URI.2\" => \"http://backup.example.com/crl\"}\n",
            "[true, 0, \"ok\", \"/DC=org/DC=fake/CN=Fake\"]\n",
            "[false, 10, \"certificate has expired\", \"/DC=org/DC=fake/CN=Fake\"]\n",
            "[false, 3, \"unable to get certificate CRL\", \"/DC=org/DC=fake/CN=Fake\"]\n",
            "[false, 23, \"certificate revoked\", \"/DC=org/DC=fake/CN=Fake\"]\n",
            "[false, 12, \"CRL has expired\", \"/DC=org/DC=fake/CN=Fake\"]\n",
            "[false, 8, \"CRL signature failure\", \"/DC=org/DC=fake/CN=Fake\"]\n",
            "2\n",
            "[\"CRLReason = Key Compromise\"]\n",
        )
    );
}

#[test]
fn openssl_to_bn_on_a_bn_and_an_integer() {
    let output = run_example("stdlib/openssl_to_bn.rb");
    assert_eq!(output, "true\ntrue\nOpenSSL::BN\n");
}

#[test]
fn openssl_ec_keys_on_a_binary_curve() {
    let output = run_example("stdlib/openssl_ec_binary_curve.rb");
    assert_eq!(
        output,
        concat!(
            "\"sect571r1\"\n",
            "571\n",
            "true\n",
            "true\n",
            "true\n",
            "true\n",
            "true\n",
            "true\n",
            "false\n",
            "true\n",
            "73\n",
            "true\n",
            "true\n",
            "true\n",
            "[\"sect571r1\", \"NIST/SECG curve over a 571 bit binary field\"]\n",
            "[\"prime256v1\", \"FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551\", \"046b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c2964fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5\", 256, 1]\n",
            "[\"secp256k1\", \"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141\", \"0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8\", 256, 1]\n",
            "[\"secp384r1\", \"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFC7634D81F4372DDF581A0DB248B0A77AECEC196ACCC52973\", \"04aa87ca22be8b05378eb1c71ef320ad746e1d3b628ba79b9859f741e082542a385502f25dbf55296c3a545e3872760ab73617de4a96262c6f5d9e98bf9292dc29f8f41dbd289a147ce9da3113b5f0b8c00a60b1ce1d7e819d7a431d7c90ea0e5f\", 384, 1]\n",
            "[\"secp521r1\", \"01FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFA51868783BF2F966B7FCC0148F709A5D03BB5C9B8899C47AEBB6FB71E91386409\", \"0400c6858e06b70404e9cd9e3ecb662395b4429c648139053fb521f828af606b4d3dbaa14b5e77efe75928fe1dc127a2ffa8de3348b3c1856a429bf97e7e31c2e5bd66011839296a789a3bc0045c8a5fb42c7d1bd998f54449579b446817afbd17273e662c97ee72995ef42640c550b9013fad0761353c7086a272c24088be94769fd16650\", 521, 1]\n",
            "[\"sect571r1\", \"03FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE661CE18FF55987308059B186823851EC7DD9CA1161DE93D5174D66E8382E9BB2FE84E47\", \"040303001d34b856296c16c0d40d3cd7750a93d1d2955fa80aa5f40fc8db7b2abdbde53950f4c0d293cdd711a35b67fb1499ae60038614f1394abfa3b4c850d927e1e7769c8eec2d19037bf27342da639b6dccfffeb73d69d78c6c27a6009cbbca1980f8533921e8a684423e43bab08a576291af8f461bb2a8b3531d2f0485c19b16e2f1516e23dd3c1a4827af1b8ac15b\", 571, 2]\n",
        )
    );
}

#[test]
fn zlib_errors_for_corrupt_and_short_streams() {
    let output = run_example("stdlib/zlib_errors_for_corrupt_and_short_streams.rb");
    assert_eq!(
        output,
        concat!(
            "\"Zlib::BufError: buffer error\"\n",
            "\"Zlib::BufError: buffer error\"\n",
            "\"Zlib::BufError: buffer error\"\n",
            "\"Zlib::BufError: buffer error\"\n",
            "\"Zlib::BufError: buffer error\"\n",
            "\"Zlib::DataError: invalid block type\"\n",
            "\"Zlib::DataError: invalid stored block lengths\"\n",
            "\"Zlib::DataError: unknown compression method\"\n",
            "\"Zlib::DataError: incorrect header check\"\n",
            "\"Zlib::DataError: incorrect data check\"\n",
            "\"Zlib::DataError: invalid block type\"\n",
            "\"Zlib::BufError: buffer error\"\n",
            "\"\"\n",
            "\"\"\n",
            "true\n",
            "480\n",
            "[480, \"\", \"\"]\n",
        )
    );
}

#[test]
fn zlib_compression_levels() {
    let output = run_example("stdlib/zlib_compression_levels.rb");
    assert_eq!(
        output,
        concat!(
            "[-1, 10722, \"789c\", \"7c4729186c7b0b46\", 10716, true, \"789c2b2c4d2c2a492dcaa95428c92f49cc29b652\"]\n",
            "[0, 90187, \"7801\", :stored, 90181, true, \"7801017b0084ff717561727465726c7920746f74\"]\n",
            "[1, 11373, \"7801\", \"37e1770d8772a627\", 11367, true, \"78012b2c4d2c2a492dcaa95428c92f49cc29b652\"]\n",
            "[2, 12114, \"785e\", \"8aefbfd79c1157de\", 12108, true, \"785e2b2c4d2c2a492dcaa95428c92f49cc29b652\"]\n",
            "[3, 11785, \"785e\", \"ae57352516b8d143\", 11779, true, \"785e2b2c4d2c2a492dcaa95428c92f49cc29b652\"]\n",
            "[4, 11273, \"785e\", \"115bda576a0242c5\", 11267, true, \"785e2b2c4d2c2a492dcaa95428c92f49cc29b652\"]\n",
            "[5, 11465, \"785e\", \"b5407aadcc939202\", 11459, true, \"785e2b2c4d2c2a492dcaa95428c92f49cc29b652\"]\n",
            "[6, 10722, \"789c\", \"7c4729186c7b0b46\", 10716, true, \"789c2b2c4d2c2a492dcaa95428c92f49cc29b652\"]\n",
            "[7, 10577, \"78da\", \"23e9579bef3dd91b\", 10571, true, \"78da2b2c4d2c2a492dcaa95428c92f49cc29b652\"]\n",
            "[8, 9018, \"78da\", \"3e6e5845ca0ffa2f\", 9012, true, \"78da2b2c4d2c2a492dcaa95428c92f49cc29b652\"]\n",
            "[9, 9018, \"78da\", \"3e6e5845ca0ffa2f\", 9012, true, \"78da2b2c4d2c2a492dcaa95428c92f49cc29b652\"]\n",
            "\"7801010000ffff00000001\"\n",
            "\"\"\n",
            "70016\n",
            "true\n",
        )
    );
}

#[test]
fn openssl_hmac_instances() {
    let output = run_example("stdlib/openssl_hmac_instances.rb");
    assert_eq!(
        output,
        concat!(
            "true\n",
            "32\n",
            "true\n",
            "true\n",
            "44\n",
            "true\n",
            "true\n",
            "false\n",
            "true\n",
            "\"4fd0b215276ef12f2b3e4c8ecac2811498b656fc\"\n",
            "ArgumentError\n",
        )
    );
}

#[test]
fn delegate_class_forwards_keyword_arguments() {
    let output = run_example("stdlib/delegate_class_forwards_keywords.rb");
    assert_eq!(
        output,
        concat!(
            "[:io, true, true]\n[:io, false, false]\n{verbose: true}\n",
            "[[1, 2], {label: \"x\"}]\n[[{a: 1}], {}]\n[:io, false, true]\n",
        )
    );
}

#[test]
fn zlib_gunzip_errors_name_what_is_wrong() {
    let output = run_example("stdlib/zlib_gunzip_errors.rb");
    assert_eq!(
        output,
        concat!(
            "\"hello\"\n",
            "\"hello\"\n",
            "[Zlib::GzipFile::Error, \"not in gzip format\"]\n",
            "[Zlib::GzipFile::Error, \"unexpected end of file\"]\n",
            "[Zlib::GzipFile::Error, \"unexpected end of string\"]\n",
            "[Zlib::GzipFile::CRCError, \"invalid compressed data -- crc error\"]\n",
            "[Zlib::GzipFile::LengthError, \"invalid compressed data -- length error\"]\n",
            "[Zlib::GzipFile::Error, \"unsupported compression method 9\"]\n",
            "[Zlib::GzipFile::Error, \"unexpected end of string\"]\n",
            "[Zlib::GzipFile::Error, \"unexpected end of string\"]\n",
            "[Zlib::GzipFile::Error, \"unexpected end of string\"]\n",
            "[Zlib::GzipFile::Error, \"unexpected end of string\"]\n",
        )
    );
}

#[test]
fn marshal_keeps_a_replaced_set_ivar() {
    let output = run_example("stdlib/marshal_keeps_a_replaced_set_ivar.rb");
    assert_eq!(output, "[\"html\", \"htm\", \"hex\"]\ntrue\n");
}

#[test]
fn marshal_links_only_the_same_object() {
    let output = run_example("stdlib/marshal_links_only_the_same_object.rb");
    assert_eq!(output, "[\"first\", \"second\", \"first\"]\ntrue\nfalse\nfalse\n");
}

#[test]
fn marshal_links_after_a_set() {
    let output = run_example("stdlib/marshal_links_after_a_set.rb");
    assert_eq!(
        output,
        "Set[\"a\", \"b\"]\nSet[\"shared\"]\n\"shared\"\ntrue\nSet[1]\ntrue\n29\n"
    );
}

#[test]
fn marshal_writes_each_set_its_own_members() {
    let output = run_example("stdlib/marshal_writes_each_set_its_own_members.rb");
    assert_eq!(output, "true\n");
}

#[test]
fn backquotes_run_shell_syntax_through_sh() {
    let output = run_example("runtime/backquotes_run_shell_syntax_through_sh.rb");
    assert_eq!(
        output,
        concat!(
            "true\n127\n",
            "[Errno::ENOENT, \"No such file or directory - missing_program_for_this_example\"]\n",
            "3\ntrue\n\"two words\\n\"\n",
        )
    );
}

#[test]
fn coverage_line_stub_marks_the_lines_that_run() {
    let output = run_example("coverage/line_stub/main.rb");
    assert_eq!(output, "[0, 0, nil, nil, 0, nil]\n:refused\n");
}

#[test]
fn coverage_keys_methods_by_their_owner_and_place() {
    let output = run_example("coverage/method_keys/main.rb");
    assert_eq!(
        output,
        "[#<Class:Ledger>, :open, 2, 2, 4, 5, 1]\n[Ledger, :balance, 6, 25, 8, 5, 2]\n[#<Class:Auditing>, :included, 12, 2, 16, 5, 1]\n[Ledger, :audited?, 14, 31, 14, 39, 1]\n[Auditing, :audit, 18, 2, 20, 5, 0]\n"
    );
}

#[test]
fn coverage_counts_empty_files_and_attribute_assignments_as_ruby_does() {
    let output = run_example("coverage/assignment_lines/main.rb");
    assert_eq!(output, "[]\n[1, 1, 1, nil, nil]\n");
}

#[test]
fn coverage_branches_are_keyed_as_ruby_keys_them() {
    let output = run_example("coverage/branches/main.rb");
    let expected = concat!(
        "[:if, 0, 2, 2, 8, 5] => {[:then, 1, 3, 4, 3, 9] => 1, [:else, 2, 4, 2, 8, 5] => 3}\n",
        "[:if, 3, 4, 2, 8, 5] => {[:then, 4, 5, 4, 5, 11] => 1, [:else, 5, 7, 4, 7, 8] => 2}\n",
        "[:unless, 6, 12, 2, 16, 5] => {[:else, 7, 15, 4, 15, 14] => 2, [:then, 8, 13, 4, 13, 12] => 1}\n",
        "[:case, 9, 20, 2, 23, 5] => {[:when, 10, 21, 19, 21, 20] => 1, [:when, 11, 22, 19, 22, 20] => 2, [:else, 12, 20, 2, 23, 5] => 1}\n",
        "[:while, 13, 30, 0, 30, 30] => {[:body, 14, 30, 0, 30, 12] => 3}\n",
        "[:\"&.\", 15, 32, 0, 32, 11] => {[:then, 16, 32, 0, 32, 11] => 0, [:else, 17, 32, 0, 32, 11] => 1}\n",
        "[:if, 18, 33, 24, 33, 50] => {[:then, 19, 33, 32, 33, 40] => 1, [:else, 20, 33, 43, 33, 50] => 1}\n",
    );
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_mkmf_running_commands_execution() {
    let output = run_example("stdlib/mkmf_running_commands.rb");
    assert_eq!(
        output,
        concat!(
            "true\ntrue\n\"maintenance window\\n\"\n\"2026-11\\n\"\n\"shell words\\n\"\n",
            "[\"\", \"\", \"\"]\ntrue\n",
            "checking for zlibVersion() in -lz... yes\ntrue\n",
            "checking for cos() in -lm... yes\ntrue\n[\"-lm\", \"-lz\"]\n",
            "[\"a\", \"o\", \"\"]\n[[], \"\"]\n",
        )
    );
}

#[test]
fn test_stdlib_openssl_pkcs5_execution() {
    let output = run_example("stdlib/openssl_pkcs5.rb");
    assert_eq!(
        output,
        concat!(
            "32\n\"186a85603697bfc938097359ccfd9b1d6ac2c75885d90ae5af8ed399eb755501\"\n",
            "true\n\"0c60c80f961f0e71f3a9b524af6012062fe037a6\"\n",
        )
    );
}

#[test]
fn test_coverage_eval_from_line_zero_execution() {
    let output = run_example("coverage/eval_from_line_zero.rb");
    assert_eq!(
        output,
        concat!(
            "{lines: [1, 4, 1, nil]}\n{lines: [0, nil]}\n",
            "{lines: [nil, nil, nil, nil, 1, 1]}\n{lines: [1, nil, 1, 1, 1, nil]}\n",
        )
    );
}

#[test]
fn test_coverage_one_line_statements_execution() {
    let output = run_example("coverage/one_line_statements/main.rb");
    assert_eq!(
        output,
        "[1, 1, 3, 1, nil, 1, 3, 1, 1, 1, 1, 2, 1, 1, 3, 1, 3, 3, 1, 1, 1, nil, 1, nil]\n"
    );
}
