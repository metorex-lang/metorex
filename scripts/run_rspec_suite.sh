#!/usr/bin/env bash
# Runs the RSpec libraries' own spec suites against Rex, each listed spec
# file in a process of its own, and prints the totals.
#
#   scripts/run_rspec_suite.sh [SUITE...]
#
# SUITE is a gem name: rspec-expectations, rspec-mocks, or rspec-core, or one
# spec file as its line below names it. With none named, every suite runs. A spec file whose line below is commented out
# does not run yet. Uncomment the next one, run, fix what fails, and repeat.
#
# The suites come from the rspec monorepo, the gems/rspec submodule, pinned
# to the release tag Rex follows. Each suite's spec directory is copied to a temporary
# directory and run there, so nothing a spec writes lands in the checkout.
# Each suite runs against Rex alone: rspec-support's spec helpers go on the
# load path, and its library does not, since Rex answers `rspec/support`.
#
# SPEC_ARGS holds arguments added to each `metorex spec` run, such as
# `--no-fast-paths`. With LOG_DIR set, what each spec file printed is copied
# there, named after the file.
#
# ENGINE=mri runs each file under MRI's own `rspec` from the submodule, with
# its four libraries on the load path, in place of Rex. With RECORD=1 the
# counts each file reached are stored as that engine's in
# scripts/rspec_suite_results.tsv. A Metorex run then lists the files whose
# counts differ from MRI's there.
#
# Needs MRI, and the gems the suites load, which
# scripts/install_rspec_suite_gems.sh installs.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
# METOREX names another binary, such as one built with `--profile suites`.
METOREX="${METOREX:-$TARGET_DIR/debug/metorex}"
RSPEC_SOURCE="$REPO_ROOT/gems/rspec"
JOBS="${JOBS:-$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)}"
# The longest one spec file may run before its process is stopped.
FILE_TIMEOUT="${FILE_TIMEOUT:-300}"
ENGINE="${ENGINE:-metorex}"
case "$ENGINE" in
  metorex | mri) ;;
  *) echo "run_rspec_suite.sh: ENGINE is metorex or mri" >&2; exit 2 ;;
esac

if [ ! -f "$METOREX" ]; then
  echo "Building metorex..."
  cargo build --manifest-path "$REPO_ROOT/Cargo.toml"
fi

if [ ! -d "$RSPEC_SOURCE/rspec-core" ]; then
  echo "gems/rspec is missing, run: git submodule update --init gems/rspec" >&2
  exit 2
fi

# The gems the suites load, which scripts/install_rspec_suite_gems.sh puts in
# RSPEC_BUNDLE, as -I arguments. The bundle stays named in the environment
# for the specs that run `bundle exec`, as it is in RSpec's own CI. The RSpec gems are left out, since Rex
# answers for them and an MRI run takes them from the submodule, and so is
# any gem metorex carries a library of its own for.
export BUNDLE_GEMFILE="$REPO_ROOT/scripts/rspec_suite/Gemfile"
export BUNDLE_PATH="${RSPEC_BUNDLE:-$TARGET_DIR/rspec_bundle}"
export BUNDLE_WITHOUT="coverage:documentation"
if ! bundle check > /dev/null 2>&1; then
  echo "run_rspec_suite.sh: the suites' gems are missing, run: scripts/install_rspec_suite_gems.sh" >&2
  exit 2
fi
# A default gem such as bundler keeps its files in MRI's own library
# directories, beside every other library MRI has, so they are copied to a
# directory of the gem's own.
BUNDLED_GEMS="$(bundle exec ruby -rfileutils -e '
  copies = ARGV[0]
  Bundler.load.specs.each do |spec|
    next if spec.name.start_with?("rspec")
    paths = spec.full_require_paths
    if spec.default_gem? && paths.none? { |path| File.exist?(path) }
      copy = File.join(copies, spec.full_name)
      Gem::Specification.load(spec.loaded_from).files.grep(%r{\Alib/}).each do |file|
        destination = File.join(copy, file.delete_prefix("lib/"))
        FileUtils.mkdir_p(File.dirname(destination))
        source = RbConfig::CONFIG.values_at("sitelibdir", "rubylibdir")
          .map { |directory| File.join(directory, file.delete_prefix("lib/")) }.find { |path| File.exist?(path) }
        FileUtils.cp(source, destination)
      end
      paths = [copy]
    end
    puts [spec.name.tr("-", "/"), *paths].join(" ")
  end
' "$TARGET_DIR/rspec_default_gems")"
BUNDLE_LOAD_PATH=""
while read -r feature paths; do
  if [ "$ENGINE" = "mri" ] || ! "$METOREX" -e "require '$feature'" > /dev/null 2>&1; then
    for path in $paths; do BUNDLE_LOAD_PATH+=" -I $path"; done
  fi
done <<< "$BUNDLED_GEMS"

# The directory by its real path, since specs compare paths they build with
# the ones Ruby reports, and macOS reaches its temporary directory through a
# link.
WORK_DIR=$(cd "$(mktemp -d -t metorex_rspec_suites.XXXXXX)" && pwd -P)
# Under a `lib` directory, as in rspec-support, since RSpec leaves frames in
# `/lib/rspec/support/` out of backtraces.
HELPERS_DIR="$WORK_DIR/support_spec_helpers/lib"
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
  if [ "${#SUITES[@]}" -eq 0 ] || [[ " ${SUITES[*]} " == *" $suite "* ]] || [[ " ${SUITES[*]} " == *" $1 "* ]]; then
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

# Copies each suite named in the queue once, all of its directory as RSpec's
# own CI runs in but its library, which Rex answers for and an MRI run takes
# from the submodule.
for path in "${SPEC_PATHS[@]}"; do
  suite="${path%%/*}"
  if [ ! -d "$WORK_DIR/$suite" ]; then
    mkdir -p "$WORK_DIR/$suite"
    for entry in "$RSPEC_SOURCE/$suite"/* "$RSPEC_SOURCE/$suite"/.[!.]*; do
      [ "$(basename "$entry")" = "lib" ] || cp -R "$entry" "$WORK_DIR/$suite/"
    done
    # An empty lib, which configuration_spec.rb sees put on the load path.
    mkdir "$WORK_DIR/$suite/lib"
  fi
done

# Runs one spec file in a copy of its suite of its own, writing what it
# printed next to it. A copy of its own keeps what one file writes, such as
# aruba's `tmp/aruba`, away from the files running beside it.
run_one() {
  local path="$1"
  local suite="${path%%/*}"
  local file="${path#*/}"
  local log="$WORK_DIR/logs/${path//\//__}.txt"
  # The copy is named after its suite, as in RSpec's own checkout, since specs
  # filter backtraces by paths such as /rspec-core/spec/.
  local run_dir="$WORK_DIR/runs/${path//\//__}/$suite"
  mkdir -p "$WORK_DIR/logs" "$run_dir"
  cp -R "$WORK_DIR/$suite/." "$run_dir/"
  local command
  if [ "$ENGINE" = "mri" ]; then
    command=(ruby -I "$RSPEC_SOURCE/rspec/lib" -I "$RSPEC_SOURCE/rspec-core/lib" -I "$RSPEC_SOURCE/rspec-expectations/lib"
      -I "$RSPEC_SOURCE/rspec-mocks/lib" -I "$RSPEC_SOURCE/rspec-support/lib"
      "$RSPEC_SOURCE/rspec-core/exe/rspec")
  else
    command=("$METOREX" spec)
  fi
  (
    cd "$run_dir"
    perl -e 'alarm shift; exec @ARGV' "$FILE_TIMEOUT" "${command[@]}" --format progress --no-color --seed 1 \
      -I "$HELPERS_DIR" $BUNDLE_LOAD_PATH ${SPEC_ARGS:-} "$file" > "$log" 2>&1
  ) || true
  local summary
  summary=$(sed 's/\x1b\[[0-9;]*m//g' "$log" | grep -E '^[0-9]+ examples?, ' | tail -1 || true)
  printf '%s\t%s\n' "$path" "${summary:-did not finish}"
}
export -f run_one
export WORK_DIR HELPERS_DIR BUNDLE_LOAD_PATH METOREX FILE_TIMEOUT SPEC_ARGS ENGINE RSPEC_SOURCE

RESULTS="$WORK_DIR/results.tsv"
# Each file's result is printed as it finishes, so a run stopped partway
# shows which files had finished.
printf 'running %s spec files, %s at a time\n' "${#SPEC_PATHS[@]}" "$JOBS"
printf '%s\n' "${SPEC_PATHS[@]}" | xargs -P "$JOBS" -I {} bash -c 'run_one "$@"' _ {} | tee "$RESULTS"
if [ -n "${LOG_DIR:-}" ]; then
  mkdir -p "$LOG_DIR"
  cp "$WORK_DIR"/logs/*.txt "$LOG_DIR/"
fi
if [ -n "${RECORD:-}" ]; then
  ruby "$REPO_ROOT/scripts/suite_results.rb" record "$ENGINE" "$RESULTS"
fi
if [ "$ENGINE" = "metorex" ]; then
  echo "Files whose counts differ from MRI's:"
  ruby "$REPO_ROOT/scripts/suite_results.rb" differences "$RESULTS" || true
fi

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
