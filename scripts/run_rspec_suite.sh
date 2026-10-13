#!/usr/bin/env bash
# Runs the RSpec libraries' own spec suites against Rex, each listed spec
# file in a process of its own, and prints the totals.
#
#   scripts/run_rspec_suite.sh [SUITE...]
#
# SUITE is a gem name: rspec-expectations, rspec-mocks, or rspec-core. With
# none named, every suite runs. A spec file whose line below is commented out
# does not run yet. Uncomment the next one, run, fix what fails, and repeat.
#
# The suites come from the rspec monorepo at the tag RSPEC_TAG, downloaded
# once into RSPEC_SOURCE. Each suite's spec directory is copied to a temporary
# directory and run there, so nothing a spec writes lands in the checkout.
# Each suite runs against Rex alone: rspec-support's spec helpers go on the
# load path, and its library does not, since Rex answers `rspec/support`.
#
# Needs MRI with the diff-lcs, rake, aruba, and thread_order gems, whose
# libraries the suites load, and curl for the first download.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
METOREX="$TARGET_DIR/debug/metorex"
RSPEC_TAG="${RSPEC_TAG:-rspec-core-v3.13.6}"
RSPEC_SOURCE="${RSPEC_SOURCE:-$TARGET_DIR/rspec_source/$RSPEC_TAG}"
JOBS="${JOBS:-$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)}"
# The longest one spec file may run before its process is stopped.
FILE_TIMEOUT="${FILE_TIMEOUT:-300}"

if [ ! -f "$METOREX" ]; then
  echo "Building metorex..."
  cargo build --manifest-path "$REPO_ROOT/Cargo.toml"
fi

if [ ! -d "$RSPEC_SOURCE" ]; then
  echo "Downloading rspec $RSPEC_TAG..."
  mkdir -p "$RSPEC_SOURCE"
  curl -sSfL "https://codeload.github.com/rspec/rspec/tar.gz/refs/tags/$RSPEC_TAG" \
    | tar -xz -C "$RSPEC_SOURCE" --strip-components 1
fi

gem_lib() {
  ruby -e "print Gem::Specification.find_by_name('$1').gem_dir" 2>/dev/null || {
    echo "run_rspec_suite.sh: MRI with the $1 gem is needed" >&2
    exit 2
  }
}
DIFF_LCS_LIB="$(gem_lib diff-lcs)/lib"
RAKE_LIB="$(gem_lib rake)/lib"
gem_lib aruba > /dev/null
gem_lib thread_order > /dev/null
# aruba, thread_order, and every gem they depend on, as -I arguments. The
# RSpec gems are left out, since Rex answers for them, and so is any gem
# metorex carries a library of its own for.
ARUBA_GEMS="$(ruby -e '
  seen = {}
  pending = %w[aruba thread_order].map { |name| Gem::Specification.find_by_name(name) }
  until pending.empty?
    spec = pending.shift
    next if seen[spec.name]
    seen[spec.name] = spec
    spec.runtime_dependencies.each { |dependency| pending << dependency.to_spec }
  end
  seen.each_value do |spec|
    next if spec.name.start_with?("rspec")
    puts [spec.name.tr("-", "/"), *spec.full_require_paths].join(" ")
  end
')"
ARUBA_LOAD_PATH=""
while read -r feature paths; do
  if ! "$METOREX" -e "require '$feature'" > /dev/null 2>&1; then
    for path in $paths; do ARUBA_LOAD_PATH+=" -I $path"; done
  fi
done <<< "$ARUBA_GEMS"

WORK_DIR=$(mktemp -d -t metorex_rspec_suites.XXXXXX)
HELPERS_DIR="$WORK_DIR/support_spec_helpers"
mkdir -p "$HELPERS_DIR/rspec/support"
cp -R "$RSPEC_SOURCE/rspec-support/lib/rspec/support/spec" \
  "$RSPEC_SOURCE/rspec-support/lib/rspec/support/spec.rb" "$HELPERS_DIR/rspec/support/"

TOTAL_FILES=0
TOTAL_EXAMPLES=0
TOTAL_FAILURES=0
TOTAL_PENDING=0
TOTAL_ERRORS=0
FILES_THAT_DID_NOT_FINISH=0

print_totals() {
  printf '\n%s files, %s examples, %s failures, %s pending, %s errors outside of examples, %s files that did not finish\n' \
    "$TOTAL_FILES" "$TOTAL_EXAMPLES" "$TOTAL_FAILURES" "$TOTAL_PENDING" "$TOTAL_ERRORS" "$FILES_THAT_DID_NOT_FINISH"
}

cleanup() {
  if [ "$TOTAL_FAILURES" != "0" ] || [ "$TOTAL_ERRORS" != "0" ] || [ "$FILES_THAT_DID_NOT_FINISH" != "0" ]; then
    printf 'what each spec file printed is in %s\n' "$WORK_DIR" >&2
    return
  fi
  rm -rf "$WORK_DIR"
}
trap 'print_totals; cleanup' EXIT

SUITES=("$@")
SPEC_PATHS=()
run_spec() {
  local suite="${1%%/*}"
  if [ "${#SUITES[@]}" -eq 0 ] || [[ " ${SUITES[*]} " == *" $suite "* ]]; then
    SPEC_PATHS+=("$1")
  fi
}

# rspec-expectations
run_spec "rspec-expectations/spec/rspec/expectations/block_snippet_extractor_spec.rb"
run_spec "rspec-expectations/spec/rspec/expectations/configuration_spec.rb"
run_spec "rspec-expectations/spec/rspec/expectations/expectation_target_spec.rb"
run_spec "rspec-expectations/spec/rspec/expectations/extensions/kernel_spec.rb"
run_spec "rspec-expectations/spec/rspec/expectations/fail_with_spec.rb"
run_spec "rspec-expectations/spec/rspec/expectations/failure_aggregator_spec.rb"
run_spec "rspec-expectations/spec/rspec/expectations/handler_spec.rb"
run_spec "rspec-expectations/spec/rspec/expectations/minitest_integration_spec.rb"
run_spec "rspec-expectations/spec/rspec/expectations/syntax_spec.rb"
run_spec "rspec-expectations/spec/rspec/expectations_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/aliased_matcher_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/aliases_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/all_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/base_matcher_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/be_between_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/be_instance_of_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/be_kind_of_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/be_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/be_within_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/captures_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/change_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/compound_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/contain_exactly_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/cover_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/eq_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/eql_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/equal_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/exist_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/has_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/have_attributes_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/include_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/match_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/operators_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/output_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/raise_error_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/respond_to_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/satisfy_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/start_and_end_with_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/throw_symbol_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/built_in/yield_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/composable_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/define_negated_matcher_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/description_generation_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/dsl_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/english_phrasing_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/legacy_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers/multi_matcher_diff_spec.rb"
run_spec "rspec-expectations/spec/rspec/matchers_spec.rb"

# rspec-mocks
run_spec "rspec-mocks/spec/integration/rails_support_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/and_call_original_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/and_invoke_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/and_return_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/and_wrap_original_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/and_yield_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/any_instance/message_chains_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/any_instance_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/argument_matchers_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/array_including_matcher_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/at_least_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/at_most_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/before_all_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/block_return_value_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/combining_implementation_instructions_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/configuration_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/diffing_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/double_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/error_generator_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/example_methods_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/expiration_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/failure_notification_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/formatting_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/hash_excluding_matcher_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/hash_including_matcher_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/instance_method_stasher_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/marshal_extension_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/matchers/have_received_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/matchers/receive_message_chain_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/matchers/receive_messages_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/matchers/receive_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/message_expectation_string_representation_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/methods_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/mock_expectation_error_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/mock_ordering_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/modifying_invoked_expectations_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/multiple_invoke_handler_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/multiple_return_value_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/mutate_const_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/mutex_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/nil_expectation_warning_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/null_object_double_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/once_counts_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/order_group_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/partial_double_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/partial_double_using_mocks_directly_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/precise_counts_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/reraising_eager_raises_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/serialization_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/should_syntax_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/space_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/spy_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/standalone_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/stash_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/stub_chain_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/stub_implementation_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/stub_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/stubbed_message_expectations_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/syntax_agnostic_message_matchers_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/syntax_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/test_double_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/thrice_counts_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/to_ary_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/twice_counts_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/verifying_doubles/class_double_with_class_loaded_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/verifying_doubles/class_double_with_class_not_loaded_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/verifying_doubles/construction_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/verifying_doubles/expected_arg_verification_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/verifying_doubles/instance_double_with_class_loaded_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/verifying_doubles/instance_double_with_class_not_loaded_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/verifying_doubles/method_visibility_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/verifying_doubles/naming_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks/verifying_doubles/object_double_spec.rb"
run_spec "rspec-mocks/spec/rspec/mocks_spec.rb"

# rspec-core
run_spec "rspec-core/spec/integration/bisect_runners_spec.rb"
run_spec "rspec-core/spec/integration/bisect_spec.rb"
run_spec "rspec-core/spec/integration/fail_if_no_examples_spec.rb"
run_spec "rspec-core/spec/integration/failed_line_detection_spec.rb"
run_spec "rspec-core/spec/integration/filtering_spec.rb"
run_spec "rspec-core/spec/integration/order_spec.rb"
run_spec "rspec-core/spec/integration/output_stream_spec.rb"
run_spec "rspec-core/spec/integration/persistence_failures_spec.rb"
run_spec "rspec-core/spec/integration/spec_file_load_errors_spec.rb"
run_spec "rspec-core/spec/integration/suite_hooks_errors_spec.rb"
run_spec "rspec-core/spec/rspec/core/aggregate_failures_spec.rb"
run_spec "rspec-core/spec/rspec/core/backtrace_formatter_spec.rb"
run_spec "rspec-core/spec/rspec/core/bisect/coordinator_spec.rb"
run_spec "rspec-core/spec/rspec/core/bisect/example_minimizer_spec.rb"
run_spec "rspec-core/spec/rspec/core/bisect/server_spec.rb"
run_spec "rspec-core/spec/rspec/core/bisect/shell_command_spec.rb"
run_spec "rspec-core/spec/rspec/core/bisect/shell_runner_spec.rb"
run_spec "rspec-core/spec/rspec/core/bisect/utilities_spec.rb"
run_spec "rspec-core/spec/rspec/core/configuration/only_failures_support_spec.rb"
run_spec "rspec-core/spec/rspec/core/configuration_options_spec.rb"
run_spec "rspec-core/spec/rspec/core/configuration_spec.rb"
run_spec "rspec-core/spec/rspec/core/did_you_mean_spec.rb"
run_spec "rspec-core/spec/rspec/core/drb_spec.rb"
run_spec "rspec-core/spec/rspec/core/dsl_spec.rb"
run_spec "rspec-core/spec/rspec/core/example_execution_result_spec.rb"
run_spec "rspec-core/spec/rspec/core/example_group_constants_spec.rb"
run_spec "rspec-core/spec/rspec/core/example_group_spec.rb"
run_spec "rspec-core/spec/rspec/core/example_spec.rb"
run_spec "rspec-core/spec/rspec/core/example_status_persister_spec.rb"
run_spec "rspec-core/spec/rspec/core/failed_example_notification_spec.rb"
run_spec "rspec-core/spec/rspec/core/filter_manager_spec.rb"
run_spec "rspec-core/spec/rspec/core/filterable_item_repository_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/base_text_formatter_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/console_codes_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/deprecation_formatter_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/documentation_formatter_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/exception_presenter_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/failure_list_formatter_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/fallback_message_formatter_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/helpers_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/html_formatter_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/html_snippet_extractor_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/json_formatter_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/profile_formatter_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/progress_formatter_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/snippet_extractor_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters/syntax_highlighter_spec.rb"
run_spec "rspec-core/spec/rspec/core/formatters_spec.rb"
run_spec "rspec-core/spec/rspec/core/hooks_filtering_spec.rb"
run_spec "rspec-core/spec/rspec/core/hooks_spec.rb"
run_spec "rspec-core/spec/rspec/core/invocations_spec.rb"
run_spec "rspec-core/spec/rspec/core/memoized_helpers_spec.rb"
run_spec "rspec-core/spec/rspec/core/metadata_filter_spec.rb"
run_spec "rspec-core/spec/rspec/core/metadata_spec.rb"
run_spec "rspec-core/spec/rspec/core/notifications_spec.rb"
run_spec "rspec-core/spec/rspec/core/option_parser_spec.rb"
run_spec "rspec-core/spec/rspec/core/ordering_spec.rb"
run_spec "rspec-core/spec/rspec/core/output_wrapper_spec.rb"
run_spec "rspec-core/spec/rspec/core/pending_example_spec.rb"
run_spec "rspec-core/spec/rspec/core/pending_spec.rb"
run_spec "rspec-core/spec/rspec/core/profiler_spec.rb"
run_spec "rspec-core/spec/rspec/core/project_initializer_spec.rb"
run_spec "rspec-core/spec/rspec/core/rake_task_spec.rb"
run_spec "rspec-core/spec/rspec/core/reporter_spec.rb"
run_spec "rspec-core/spec/rspec/core/resources/a_spec.rb"
run_spec "rspec-core/spec/rspec/core/resources/acceptance/foo_spec.rb"
run_spec "rspec-core/spec/rspec/core/rspec_matchers_spec.rb"
run_spec "rspec-core/spec/rspec/core/ruby_project_spec.rb"
run_spec "rspec-core/spec/rspec/core/runner_spec.rb"
run_spec "rspec-core/spec/rspec/core/set_spec.rb"
run_spec "rspec-core/spec/rspec/core/shared_context_spec.rb"
run_spec "rspec-core/spec/rspec/core/shared_example_group_spec.rb"
run_spec "rspec-core/spec/rspec/core/suite_hooks_spec.rb"
run_spec "rspec-core/spec/rspec/core/warnings_spec.rb"
run_spec "rspec-core/spec/rspec/core/world_spec.rb"
run_spec "rspec-core/spec/rspec/core_spec.rb"

# Copies each suite named in the queue once.
for path in "${SPEC_PATHS[@]}"; do
  suite="${path%%/*}"
  if [ ! -d "$WORK_DIR/$suite" ]; then
    mkdir -p "$WORK_DIR/$suite"
    cp -R "$RSPEC_SOURCE/$suite/spec" "$WORK_DIR/$suite/"
    cp "$RSPEC_SOURCE/$suite/.rspec" "$WORK_DIR/$suite/" 2>/dev/null || true
  fi
done

# Runs one spec file in its suite's copy, writing what it printed next to it.
run_one() {
  local path="$1"
  local suite="${path%%/*}"
  local file="${path#*/}"
  local log="$WORK_DIR/logs/${path//\//__}.txt"
  mkdir -p "$WORK_DIR/logs"
  (
    cd "$WORK_DIR/$suite"
    perl -e 'alarm shift; exec @ARGV' "$FILE_TIMEOUT" "$METOREX" spec --format progress --no-color --seed 1 \
      -I "$HELPERS_DIR" -I "$DIFF_LCS_LIB" -I "$RAKE_LIB" $ARUBA_LOAD_PATH "$file" > "$log" 2>&1
  ) || true
  local summary
  summary=$(sed 's/\x1b\[[0-9;]*m//g' "$log" | grep -E '^[0-9]+ examples?, ' | tail -1 || true)
  printf '%s\t%s\n' "$path" "${summary:-did not finish}"
}
export -f run_one
export WORK_DIR HELPERS_DIR DIFF_LCS_LIB RAKE_LIB ARUBA_LOAD_PATH METOREX FILE_TIMEOUT

RESULTS="$WORK_DIR/results.tsv"
printf '%s\n' "${SPEC_PATHS[@]}" | xargs -P "$JOBS" -I {} bash -c 'run_one "$@"' _ {} > "$RESULTS"

while IFS=$'\t' read -r path summary; do
  TOTAL_FILES=$((TOTAL_FILES + 1))
  if [ "$summary" = "did not finish" ]; then
    FILES_THAT_DID_NOT_FINISH=$((FILES_THAT_DID_NOT_FINISH + 1))
    printf 'did not finish: %s\n' "$path"
    continue
  fi
  examples=$(sed -E 's/^([0-9]+) examples?.*/\1/' <<< "$summary")
  failures=$(sed -E 's/.* ([0-9]+) failures?.*/\1/' <<< "$summary")
  pending=0
  errors=0
  if [[ "$summary" =~ ([0-9]+)\ pending ]]; then pending="${BASH_REMATCH[1]}"; fi
  if [[ "$summary" =~ ([0-9]+)\ errors?\ occurred ]]; then errors="${BASH_REMATCH[1]}"; fi
  TOTAL_EXAMPLES=$((TOTAL_EXAMPLES + examples))
  TOTAL_FAILURES=$((TOTAL_FAILURES + failures))
  TOTAL_PENDING=$((TOTAL_PENDING + pending))
  TOTAL_ERRORS=$((TOTAL_ERRORS + errors))
  if [ "$failures" != "0" ] || [ "$errors" != "0" ]; then
    printf '%s: %s\n' "$path" "$summary"
  fi
done < <(sort "$RESULTS")
