// Examples covering StringScanner

use super::run_example;

/// The expected output of both `scanning/reading_a_string` variants, which
/// differ only in whether the calls are written with parentheses.
const READING_A_STRING_OUTPUT: &str = "\"This\"\n4\n\" \"\nnil\n\"is\"\n5\n2\n2\n\" a test\"\n7\nfalse\n0\n\"This\"\n\"Th\"\n\" is a test\"\n\"is\"\ntrue\n2\n0\n\"This\"\n10\n\" is a\"\n5\n\" test\"\n\"2024-05\"\n\"2024-05\"\n\"2024\"\n\"2024\"\n[\"2024\", \"05\"]\n{\"year\" => \"2024\", \"month\" => \"05\"}\n[\"2024-05\", \"2024\", \"05\"]\n3\n\"a\"\n\"bc\"\n98\n98\nfalse\ntrue\n\"#<StringScanner fin>\"\nStringScanner::Error\n";

#[test]
fn test_scanning_reading_a_string_execution() {
    let output = run_example("scanning/reading_a_string.rb");
    assert_eq!(output, READING_A_STRING_OUTPUT);
}

#[test]
fn test_scanning_reading_a_string_no_parens_execution() {
    let output = run_example("scanning/reading_a_string_no_parens.rb");
    assert_eq!(output, READING_A_STRING_OUTPUT);
}

/// The expected output of both `scanning/byte_positions/cursor` variants.
const BYTE_POSITIONS_OUTPUT: &str = concat!(
    "10\n",
    "10\n",
    "8\n",
    "3\n",
    "\"ädeföghi\"\n",
    "\"T\"\n",
    "\"T\"\n",
    "true\n",
    "\"東京\"\n",
    "[230, 157, 177, 228, 186, 172]\n"
);

#[test]
fn test_scanning_byte_positions_cursor_execution() {
    let output = run_example("scanning/byte_positions/cursor.rb");
    assert_eq!(output, BYTE_POSITIONS_OUTPUT);
}

#[test]
fn test_scanning_byte_positions_cursor_parens_execution() {
    let output = run_example("scanning/byte_positions/cursor_parens.rb");
    assert_eq!(output, BYTE_POSITIONS_OUTPUT);
}

/// The expected output of both `scanning/buffered_stream/reading` variants.
const BUFFERED_STREAM_OUTPUT: &str = concat!(
    "true\n",
    "\"this\"\n",
    "\"this is\\n\"\n",
    "\"this is\\n\"\n",
    "\"an example\"\n",
    "[\"this \", \"is\\nan \", \"example\\n\"]\n",
    "[\"this is\\n\", \"an example\\n\"]\n",
    "\"1234\"\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:US-ASCII>\n"
);

#[test]
fn test_scanning_buffered_stream_reading_execution() {
    let output = run_example("scanning/buffered_stream/reading.rb");
    assert_eq!(output, BUFFERED_STREAM_OUTPUT);
}

#[test]
fn test_scanning_buffered_stream_reading_parens_execution() {
    let output = run_example("scanning/buffered_stream/reading_parens.rb");
    assert_eq!(output, BUFFERED_STREAM_OUTPUT);
}

/// The expected output of both `scanning/repeated_matches` variants, which
/// differ only in whether the calls are written with parentheses.
const REPEATED_MATCHES_OUTPUT: &str = concat!(
    "[\"cruel\", \"world\"]\n",
    "[[\"cr\", \"ue\"], [\"l \", \"wo\"]]\n",
    "[\"\", \"\", \"\", \"\", \"\", \"\"]\n",
    "[\"one\"]\n",
    "[\"one \", \"two \", \"one \", \"two\"]\n",
    "[\"o\", \"o\"]\n",
    "[[\"a\", \"b\"]]\n",
    "3\n",
    "nil\n"
);

#[test]
fn test_scanning_repeated_matches_execution() {
    let output = run_example("scanning/repeated_matches.rb");
    assert_eq!(output, REPEATED_MATCHES_OUTPUT);
}

#[test]
fn test_scanning_repeated_matches_parens_execution() {
    let output = run_example("scanning/repeated_matches_parens.rb");
    assert_eq!(output, REPEATED_MATCHES_OUTPUT);
}

/// The expected output of both `scanning/substitutions` variants, which
/// differ only in whether the calls are written with parentheses.
const SUBSTITUTIONS_OUTPUT: &str = concat!(
    "\"heLlo\"\n",
    "\"heLLo\"\n",
    "\"heLLo\"\n",
    "\"he<l><l>o\"\n",
    "\"hlelo\"\n",
    "\"h311o\"\n",
    "\" Text\\n\"\n",
    "\" Text\\n Foo\"\n",
    "[\"a\", \"a\"]\n",
    "nil\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "\"heyyo\"\n",
    "nil\n",
    "\"l\"\n",
    "\"hello\"\n"
);

#[test]
fn test_scanning_substitutions_execution() {
    let output = run_example("scanning/substitutions.rb");
    assert_eq!(output, SUBSTITUTIONS_OUTPUT);
}

#[test]
fn test_scanning_substitutions_parens_execution() {
    let output = run_example("scanning/substitutions_parens.rb");
    assert_eq!(output, SUBSTITUTIONS_OUTPUT);
}

#[test]
fn the_searching_methods_look_for_a_string_as_its_text() {
    let output = run_example("scanning/searching_for_a_string.rb");
    assert_eq!(
        output,
        "\"abc literal\"\n11\n\"literal\"\n\"abc \"\n\"literal\"\nnil\n1\n\"x\"\n1\nnil\n\"a.\"\n2\n"
    );
}

#[test]
fn a_scanner_reads_bytes_inside_a_character() {
    let output = run_example("scanning/bytes_inside_a_character.rb");
    assert_eq!(
        output,
        "\"\\xE2\"\n1\n\"\\xE2\"\n\"\\x88\"\n\"\\x82x\"\n2\n1\n136\n136\n2\n\"\\x88\"\nfalse\n\"\\x82x\"\n"
    );
}

#[test]
fn a_scanner_placed_at_a_byte_offset_reads_from_there() {
    let output = run_example("scanning/byte_position_inside_multibyte_text.rb");
    assert_eq!(
        output,
        "[0, 0, 0, \"aé日本x\"]\n[1, 1, 1, \"é日本x\"]\n[3, 3, 2, \"日本x\"]\n[4, 4, 3, \"\\x97\\xA5本x\"]\n[6, 6, 3, \"本x\"]\n[9, 9, 4, \"x\"]\n[10, 10, 5, \"\"]\n"
    );
}

#[test]
fn test_scanning_appending_to_the_scanned_string_execution() {
    let output = run_example("scanning/appending_to_the_scanned_string.rb");
    assert_eq!(output, "\"abcd\"\ntrue\nFrozenError\n");
}
