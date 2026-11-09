use super::run_example;

/// The expected output of both `regexp/matching` variants, which differ only
/// in whether the calls are written with parentheses.
const MATCHING_OUTPUT: &str = "MatchData\n\"hello there\"\n\"hello\"\n\"there\"\n[\"hello\", \"there\"]\n[\"hello there\", \"hello\", \"there\"]\n\"\"\n\" world\"\n0\n11\n[6, 11]\n3\n\"hello there\"\n\"hello\"\n5\n[\"hello there\", \"there\", nil]\n\"hello there world\"\n\"hello\"\n\"hello\"\n[\"greeting\"]\n{\"greeting\" => \"hello\"}\n[\"hello\"]\n\"#<MatchData \\\"hello there\\\" greeting:\\\"hello\\\">\"\nnil\n\"index 9 out of matches\"\n\"the quick\"\n\"quick\"\n\"the quick\"\n\"\"\n\"quick\"\nnil\nnil\n\"ab+c\"\n1\ntrue\n\"ab\"\n\"cat|dog\"\ntrue\nRegexp\ntrue\ntrue\n[4]\n[\"apple\"]\n[\"grape\"]\n";

#[test]
fn test_regexp_matching_execution() {
    let output = run_example("regexp/matching.rb");
    assert_eq!(output, MATCHING_OUTPUT);
}

#[test]
fn test_regexp_matching_no_parens_execution() {
    let output = run_example("regexp/matching_no_parens.rb");
    assert_eq!(output, MATCHING_OUTPUT);
}

/// The expected output of both `regexp/named_groups` variants, which differ
/// only in whether the calls are written with parentheses.
const NAMED_GROUPS_OUTPUT: &str = "\"113\"\n3\n6\n[\"a\"]\n{\"a\" => \"113\"}\n\"H\"\n1\n[\"æ\", \"b\"]\n[\"a\", \"b\"]\n{\"a\" => [1, 3], \"b\" => [2]}\ntrue\ntrue\nfalse\n/\\[/\n\"[\"\n\"o w\"\n\"w\"\ntrue\ntrue\n\"hi\"\n[104, 105]\n[104, 233]\n(19022/1)\n(-190227/10)\nInfinity\n-Infinity\n50.0\n";

#[test]
fn test_regexp_named_groups_execution() {
    let output = run_example("regexp/named_groups.rb");
    assert_eq!(output, NAMED_GROUPS_OUTPUT);
}

#[test]
fn test_regexp_named_groups_no_parens_execution() {
    let output = run_example("regexp/named_groups_no_parens.rb");
    assert_eq!(output, NAMED_GROUPS_OUTPUT);
}

/// The expected output of both `regexp/pattern_readings` variants.
const PATTERN_READINGS_OUTPUT: &str = "\"\\\\*\\\\?\\\\{\\\\}\\\\.\\\\+\\\\^\\\\$\\\\[\\\\]\\\\(\\\\)\\\\-\\\\ \"\n\"\\\\n\\\\r\\\\f\\\\t\"\n#<Encoding:US-ASCII>\n\"a\\\\.b\"\n\"(?i-mx:nothing)\"\n\"(?i-mx:abc)\"\n\"(?-mix:abc)\"\n\"@\"\n\"\\\\+\"\n#<Encoding:US-ASCII>\n#<Encoding:UTF-8>\n/foo/\nnil\ntrue\nfalse\n[1, 2, 4]\nfalse\ntrue\n:\"foo bar\"\n";

#[test]
fn test_regexp_pattern_readings_execution() {
    let output = run_example("regexp/pattern_readings.rb");
    assert_eq!(output, PATTERN_READINGS_OUTPUT);
}

#[test]
fn test_regexp_pattern_readings_no_parens_execution() {
    let output = run_example("regexp/pattern_readings_no_parens.rb");
    assert_eq!(output, PATTERN_READINGS_OUTPUT);
}

/// The expected output of both `regexp/pattern_options` variants, which differ only in whether
/// the calls are written with parentheses.
const PATTERN_OPTIONS_OUTPUT: &str = "0\n1\n2\n4\n7\n16\n16\n16\n0\n32\n\"uninitialized Regexp\"\n";

#[test]
fn test_regexp_pattern_options_execution() {
    let output = run_example("regexp/pattern_options.rb");
    assert_eq!(output, PATTERN_OPTIONS_OUTPUT);
}

#[test]
fn test_regexp_pattern_options_parens_execution() {
    let output = run_example("regexp/pattern_options_parens.rb");
    assert_eq!(output, PATTERN_OPTIONS_OUTPUT);
}

/// The expected output of both `regexp/end_of_subject` variants, which show
/// where `\z` and `\Z` stand in a subject and differ only in whether the calls are
/// written with parentheses.
const END_OF_SUBJECT_OUTPUT: &str = "2\nnil\n2\n2\n\"file\"\n0\n";

#[test]
fn test_regexp_end_of_subject_execution() {
    let output = run_example("regexp/end_of_subject.rb");
    assert_eq!(output, END_OF_SUBJECT_OUTPUT);
}

#[test]
fn test_regexp_end_of_subject_no_parens_execution() {
    let output = run_example("regexp/end_of_subject_no_parens.rb");
    assert_eq!(output, END_OF_SUBJECT_OUTPUT);
}

/// The expected output of both `regexp/built_patterns` variants, which differ
/// only in whether the calls are written with parentheses.
const BUILT_PATTERNS_OUTPUT: &str = concat!(
    "1\n",
    "7\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:UTF-8>\n",
    "unknown regexp option: e\n",
    "premature end of char-class: /^[$/\n",
    "no implicit conversion of Symbol into String\n",
    "true\n",
    "false\n"
);

#[test]
fn test_regexp_built_patterns_execution() {
    let output = run_example("regexp/built_patterns.rb");
    assert_eq!(output, BUILT_PATTERNS_OUTPUT);
}

#[test]
fn test_regexp_built_patterns_parens_execution() {
    let output = run_example("regexp/built_patterns_parens.rb");
    assert_eq!(output, BUILT_PATTERNS_OUTPUT);
}

/// The expected output of both `regexp/pattern_soundness` variants, which
/// differ only in whether the calls are written with parentheses.
const PATTERN_SOUNDNESS_OUTPUT: &str = concat!(
    "end pattern with unmatched parenthesis: /(hay(st)ack/\n",
    "unmatched close parenthesis: /hay)stack/\n",
    "invalid group name <1a>: /(?<1a>a)/\n",
    "invalid group name <-a>: /(?<-a>a)/\n",
    "unknown regexp option: n\n",
    "\"Hi\"\n"
);

#[test]
fn test_regexp_pattern_soundness_execution() {
    let output = run_example("regexp/pattern_soundness.rb");
    assert_eq!(output, PATTERN_SOUNDNESS_OUTPUT);
}

#[test]
fn test_regexp_pattern_soundness_parens_execution() {
    let output = run_example("regexp/pattern_soundness_parens.rb");
    assert_eq!(output, PATTERN_SOUNDNESS_OUTPUT);
}

/// The expected output of both `regexp/backtracking_engine` variants, which
/// differ only in whether the calls are written with parentheses.
const BACKTRACKING_ENGINE_OUTPUT: &str = concat!(
    "\"abab\"\n\"abab\"\n",
    "\"bar\"\n\"bar\"\n",
    "nil\nnil\n\"aaa\"\n",
    "\"right\"\n\"aa\"\n",
    "nil\n0\n",
    "\"🤘🏽\"\n"
);

#[test]
fn test_regexp_backtracking_engine_execution() {
    let output = run_example("regexp/backtracking_engine.rb");
    assert_eq!(output, BACKTRACKING_ENGINE_OUTPUT);
}

#[test]
fn test_regexp_backtracking_engine_no_parens_execution() {
    let output = run_example("regexp/backtracking_engine_no_parens.rb");
    assert_eq!(output, BACKTRACKING_ENGINE_OUTPUT);
}

/// The expected output of both `regexp/pattern_time_limits` variants, which differ only in whether the
/// calls are written with parentheses.
const PATTERN_TIME_LIMITS_OUTPUT: &str = concat!(
    "nil\n",
    "0.5\n",
    "\"(a*)*b\"\n",
    "nil\n",
    "\"invalid timeout: 0\"\n",
    "\"invalid timeout: -1\"\n",
    "5.0\n",
    "nil\n",
    "\"aaa\"\n",
    "nil\n",
    "RegexpError\n",
);

#[test]
fn test_regexp_pattern_time_limits_execution() {
    let output = run_example("regexp/pattern_time_limits.rb");
    assert_eq!(output, PATTERN_TIME_LIMITS_OUTPUT);
}

#[test]
fn test_regexp_pattern_time_limits_parens_execution() {
    let output = run_example("regexp/pattern_time_limits_parens.rb");
    assert_eq!(output, PATTERN_TIME_LIMITS_OUTPUT);
}

/// The expected output of both `regexp/match_dispatch` variants, which differ
/// only in whether the calls are written with parentheses.
const MATCH_DISPATCH_OUTPUT: &str = concat!(
    "type mismatch: String given\n",
    "matched w00t\n",
    "ll\n",
    "asked hello\n",
    "asked hello\n",
);

#[test]
fn test_regexp_match_dispatch_execution() {
    let output = run_example("regexp/match_dispatch.rb");
    assert_eq!(output, MATCH_DISPATCH_OUTPUT);
}

#[test]
fn test_regexp_match_dispatch_no_parens_execution() {
    let output = run_example("regexp/match_dispatch_no_parens.rb");
    assert_eq!(output, MATCH_DISPATCH_OUTPUT);
}

/// The expected output of both `regexp/percent_r_and_conditions` variants,
/// which differ only in whether the calls are written with parentheses.
const PERCENT_R_AND_CONDITIONS_OUTPUT: &str = concat!(
    " () [c]{1} \n",
    "(?-mix:\\/)\n",
    "SyntaxError\n",
    "SyntaxError\n",
    "SyntaxError\n",
    "[\"foo1barfoo2\", \"foo2\"]\n",
    "true\n",
    "nil\n",
);

#[test]
fn test_regexp_percent_r_and_conditions_execution() {
    let output = run_example("regexp/percent_r_and_conditions.rb");
    assert_eq!(output, PERCENT_R_AND_CONDITIONS_OUTPUT);
}

#[test]
fn test_regexp_percent_r_and_conditions_parens_execution() {
    let output = run_example("regexp/percent_r_and_conditions_parens.rb");
    assert_eq!(output, PERCENT_R_AND_CONDITIONS_OUTPUT);
}

/// The expected output of both `regexp/pattern_unions` variants.
const PATTERN_UNIONS_OUTPUT: &str = concat!(
    "/(?!)/\n",
    "/n|\\./\n",
    "/(?-mix:dogs)|(?i-mx:cats)/\n",
    "/(?-mix:dogs)|(?i-mx:cats)/\n",
    "/foo/\n",
    "true\n",
    "/from_regexp/\n",
    "/(?-mix:from_regexp)|bar/\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:ISO-8859-1>\n",
    "#<Encoding:UTF-16LE>\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "#<Encoding:UTF-8>\n",
    "[ArgumentError, \"incompatible encodings: UTF-16LE and UTF-16BE\"]\n",
    "[ArgumentError, \"incompatible encodings: ISO-8859-1 and UTF-8\"]\n",
    "[ArgumentError, \"ASCII incompatible encoding: UTF-16LE\"]\n",
    "[TypeError, \"no implicit conversion of Array into String\"]\n"
);

#[test]
fn test_regexp_pattern_unions_execution() {
    let output = run_example("regexp/pattern_unions.rb");
    assert_eq!(output, PATTERN_UNIONS_OUTPUT);
}

#[test]
fn test_regexp_pattern_unions_parens_execution() {
    let output = run_example("regexp/pattern_unions_parens.rb");
    assert_eq!(output, PATTERN_UNIONS_OUTPUT);
}

/// The expected output of both `regexp/regexp_subclass_dispatch` variants.
const REGEXP_SUBCLASS_DISPATCH_OUTPUT: &str = "match [\"meter\"]\n\"me\"\n\"logged:m.\"\nmatch [\"meter\"]\n\"me\"\nfalse\n\"already initialized regexp\"\n";

#[test]
fn test_regexp_regexp_subclass_dispatch_execution() {
    let output = run_example("regexp/regexp_subclass_dispatch.rb");
    assert_eq!(output, REGEXP_SUBCLASS_DISPATCH_OUTPUT);
}

#[test]
fn test_regexp_regexp_subclass_dispatch_no_parens_execution() {
    let output = run_example("regexp/regexp_subclass_dispatch_no_parens.rb");
    assert_eq!(output, REGEXP_SUBCLASS_DISPATCH_OUTPUT);
}

#[test]
fn unicode_block_and_age_properties_match_their_characters() {
    let output = run_example("regexp/unicode_block_and_age_classes.rb");
    assert_eq!(
        output,
        "true\ntrue\ntrue\ntrue\nfalse\ntrue\nfalse\ntrue\nfalse\ntrue\n"
    );
}

#[test]
fn the_last_match_belongs_to_the_method_frame_that_set_it() {
    let output = run_example("regexp/last_match_belongs_to_its_method.rb");
    assert_eq!(
        output,
        "[\"Net::IMAP::ResponseParseError\", [\"1042\", \"7\"]]\n\"gam\"\n\"x\"\nnil\n\"1042\"\nnil\n[\"w\", \"a\", \"w\", \"a\"]\n[\"w\", \"a\", \"w\", \"a\"]\n[\"w\", \"a\", \"w\", \"a\"]\n"
    );
}

#[test]
fn a_block_sets_the_last_match_of_the_frame_it_was_written_in_while_a_method_runs_it() {
    let output = run_example("regexp/last_match_set_by_a_block_inside_a_call.rb");
    assert_eq!(output, "nil\n\"42\"\n\"A7\"\n\"9\"\n");
}

#[test]
fn a_unicode_escape_may_list_several_code_points() {
    let output = run_example("regexp/unicode_escape_with_several_code_points.rb");
    assert_eq!(
        output,
        "true\ntrue\ntrue\ntrue\nfalse\n\"aBi̇c\"\n\"ABC\"\n\"😀 A\"\ntrue\n\"\\\\u{41 42}\"\n\"AB\"\n\"ABB\"\ntrue\n"
    );
}

#[test]
fn inline_flags_reach_only_to_the_end_of_their_group() {
    let output = run_example("regexp/inline_flags_turn_extended_mode_off.rb");
    assert_eq!(output, "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n");
}

#[test]
fn a_repeat_led_by_one_character_reads_any_length_of_text() {
    let output = run_example("regexp/long_single_character_repeats.rb");
    assert_eq!(
        output,
        "0\n\"root\"\n50001\nnil\n\"b\"\n\"aaa\"\n\"aa\"\n\"aa\"\nnil\n\"a\\nb\"\n0\n\"ab&amp;cd%x;e\"\n\"x\"\n\"amp\"\n\"aab\"\n\"aaa\"\nnil\n"
    );
}

#[test]
fn a_pattern_matches_an_object_through_its_to_str() {
    let output = run_example("regexp/match_against_an_object_with_to_str.rb");
    assert_eq!(output, "0\n#<MatchData \"blah\">\ntrue\ntrue\n0\n");
}

#[test]
fn match_on_nil_leaves_its_block_to_no_later_call() {
    let output = run_example("regexp/match_nil_with_a_block.rb");
    assert_eq!(output, "[nil, true]\n[\"utf-8\", true]\n");
}

#[test]
fn a_pattern_opening_with_any_run_finds_the_leftmost_matches() {
    let output = run_example("regexp/leading_any_run.rb");
    assert_eq!(
        output,
        concat!(
            "[\"xy\"]\n[\"ab\\ncd\\nxy\"]\n0\n\"line2 key\"\n1\n",
            "\"start --B\\r\\nhead\\r\\n\"\n0\nnil\nnil\n",
        )
    );
}

#[test]
fn each_block_match_of_gsub_and_scan_names_its_own_match_data() {
    let output = run_example("regexp/match_data_of_each_block_match.rb");
    assert_eq!(
        output,
        concat!(
            "\"=, =, =\"\n",
            "[0, \"one\", 4, true, false]\n",
            "[7, \"two\", 11, true, false]\n",
            "[14, \"three\", 20, true, false]\n",
            "1\n",
            "true\n",
            "\"one=1, two=2, three=3\"\n",
            "\"3\"\n",
            "[15, 29, \"clair\", \"pastry\"]\n",
        )
    );
}

#[test]
fn a_regexp_literal_is_frozen_and_one_built_by_regexp_new_freezes_when_asked() {
    let output = run_example("regexp/regexp_freeze_and_frozen.rb");
    assert_eq!(output, "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\n");
}

#[test]
fn a_lookbehind_reads_back_only_as_far_as_it_can_match() {
    let output = run_example("regexp/lookbehind_reads_back_only_as_far_as_it_can_match.rb");
    assert_eq!(output, "\"abNcd\\r\\nefN\"\n2\ntrue\n7\n");
}

#[test]
fn repeated_alternatives_fail_once_per_place() {
    let output = run_example("regexp/repeated_alternatives_fail_once_per_place.rb");
    assert_eq!(
        output,
        concat!(
            "true\n",
            "false\n",
            "0\n",
            "nil\n",
            "\"a=\"\n",
            "7998\n",
            "\"\"\n",
            "\"aaab\"\n",
            "\"ab\"\n",
            "\"aaay\"\n",
            "[\"a\", \"a\", \"a\"]\n",
        )
    );
}

#[test]
fn whole_pattern_calls() {
    let output = run_example("regexp/whole_pattern_calls.rb");
    assert_eq!(
        output,
        "\"(a(b)c)\"\n\"[\\\"#<Class:#<Object:0x1>>\\\", :m]\"\n"
    );
}

#[test]
fn subclass_patterns_match_strings() {
    let output = run_example("regexp/subclass_patterns_match_strings.rb");
    assert_eq!(output, "true\nfalse\n");
}
