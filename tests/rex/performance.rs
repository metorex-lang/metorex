// The fast paths: each runs while the methods it stands in for are Rex's own,
// and a fixture that redefines one of them proves the Ruby path runs instead.

use super::run_spec;

/// The summary line of `metorex spec` run with `arguments` in `fixture`.
fn summary(fixture: &str, arguments: &[&str]) -> String {
    run_spec(fixture, arguments)
        .output
        .lines()
        .find(|line| line.contains(" examples, ") || line.contains(" example, "))
        .unwrap_or_default()
        .to_string()
}

#[test]
fn test_hooks_run_in_rspec_s_order_from_the_flattened_list() {
    assert_eq!(
        summary("performance/flattened_hooks", &["spec/flattened_spec.rb"]),
        "2 examples, 0 failures"
    );
}

#[test]
fn test_hooks_run_in_rspec_s_order_with_fast_paths_off() {
    assert_eq!(
        summary(
            "performance/flattened_hooks",
            &["--no-fast-paths", "spec/flattened_spec.rb", "-e", "order"]
        ),
        "1 example, 0 failures"
    );
}

#[test]
fn test_a_redefined_hook_walk_runs_in_place_of_the_flattened_list() {
    assert_eq!(
        summary("performance/flattened_hooks", &["spec/redefined_spec.rb"]),
        "1 example, 0 failures"
    );
}

#[test]
fn test_a_let_read_again_comes_from_its_slot() {
    assert_eq!(
        summary("performance/let_slots", &["spec/let_slots_spec.rb"]),
        "1 example, 0 failures"
    );
}

#[test]
fn test_a_let_with_fast_paths_off_asks_the_memo_every_time() {
    assert_eq!(
        summary(
            "performance/let_slots",
            &["--no-fast-paths", "spec/let_slots_spec.rb"]
        ),
        "1 example, 1 failure"
    );
}

#[test]
fn test_a_redefined_memo_lookup_answers_every_read_of_a_let() {
    assert_eq!(
        summary("performance/let_slots", &["spec/redefined_spec.rb"]),
        "1 example, 0 failures"
    );
}

#[test]
fn test_an_example_s_metadata_is_filled_natively_and_reads_as_rspec_writes_it() {
    assert_eq!(
        summary(
            "performance/native_metadata",
            &["spec/native_metadata_spec.rb"]
        ),
        "3 examples, 0 failures"
    );
}

#[test]
fn test_with_fast_paths_off_metadata_is_filled_by_the_ruby_steps() {
    assert_eq!(
        summary(
            "performance/native_metadata",
            &["--no-fast-paths", "spec/native_metadata_spec.rb"]
        ),
        "3 examples, 1 failure"
    );
}

#[test]
fn test_a_redefined_metadata_step_fills_the_metadata() {
    assert_eq!(
        summary("performance/native_metadata", &["spec/redefined_spec.rb"]),
        "1 example, 0 failures"
    );
}

#[test]
fn test_example_group_instances_are_built_without_running_initialize() {
    assert_eq!(
        summary(
            "performance/native_group_instances",
            &["spec/native_instances_spec.rb"]
        ),
        "3 examples, 0 failures"
    );
}

#[test]
fn test_with_fast_paths_off_example_group_instances_run_initialize() {
    assert_eq!(
        summary(
            "performance/native_group_instances",
            &["--no-fast-paths", "spec/native_instances_spec.rb"]
        ),
        "3 examples, 1 failure"
    );
}

#[test]
fn test_a_group_that_defines_initialize_runs_it() {
    assert_eq!(
        summary(
            "performance/native_group_instances",
            &["spec/own_initialize_spec.rb"]
        ),
        "1 example, 0 failures"
    );
}

#[test]
fn test_the_native_matchers_match_without_running_matches() {
    assert_eq!(
        summary(
            "performance/native_matchers",
            &["spec/native_matchers_spec.rb"]
        ),
        "2 examples, 0 failures"
    );
}

#[test]
fn test_with_fast_paths_off_each_matcher_runs_matches() {
    assert_eq!(
        summary(
            "performance/native_matchers",
            &["--no-fast-paths", "spec/native_matchers_spec.rb"]
        ),
        "2 examples, 1 failure"
    );
}

#[test]
fn test_a_redefined_matches_runs_for_each_native_matcher() {
    assert_eq!(
        summary("performance/native_matchers", &["spec/redefined_spec.rb"]),
        "1 example, 0 failures"
    );
}

#[test]
fn test_an_example_with_no_doubles_skips_its_mock_space() {
    assert_eq!(
        summary("performance/idle_mock_space", &["spec/idle_space_spec.rb"]),
        "5 examples, 0 failures"
    );
}

#[test]
fn test_with_fast_paths_off_every_mock_space_is_walked() {
    assert_eq!(
        summary(
            "performance/idle_mock_space",
            &["--no-fast-paths", "spec/idle_space_spec.rb"]
        ),
        "5 examples, 1 failure"
    );
}

#[test]
fn test_a_redefined_reset_all_takes_down_every_example_s_space() {
    assert_eq!(
        summary("performance/idle_mock_space", &["spec/redefined_spec.rb"]),
        "2 examples, 0 failures"
    );
}
