#!/usr/bin/env bash
# Runs the RSpec libraries' own suites twice through scripts/run_rspec_suite.sh,
# once as they run by default and once with `--no-fast-paths`, and compares
# what each spec file printed byte for byte. The lines that change from run to
# run are written alike first: the timing line, durations, object addresses,
# a seed a new configuration picks, and the names of temporary directories,
# with what is left of an address that a long inspected value was cut short
# in the middle of.
# So is the `fast_paths: false` an inspected configuration shows in the second
# run, and the backtrace lines in Rex's own files and the RSpec files Rex
# carries are left out. Exits 1 when any file printed something different.
#
#   scripts/compare_fast_paths.sh [SUITE...]
#
# The SUITE names are handed to run_rspec_suite.sh.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
COMPARE_DIR=$(mktemp -d -t metorex_fast_paths.XXXXXX)

# A run that fails prints what it wrote, which is otherwise kept only in
# the temporary directory.
run_suites() {
  local log="$COMPARE_DIR/$1.txt"
  shift
  if ! "$@" > "$log" 2>&1; then
    printf 'scripts/run_rspec_suite.sh failed, having printed:\n' >&2
    cat "$log" >&2
    exit 1
  fi
}

run_suites on env LOG_DIR="$COMPARE_DIR/on" "$REPO_ROOT/scripts/run_rspec_suite.sh" "$@"
run_suites off env LOG_DIR="$COMPARE_DIR/off" SPEC_ARGS=--no-fast-paths \
  "$REPO_ROOT/scripts/run_rspec_suite.sh" "$@"

normalized() {
  # A backtrace line in Rex's own code is left out: a fast path runs Rex's
  # work through other methods, and RSpec's default backtrace filter hides
  # those frames, which the suites' helpers show in full.
  grep -v -E '^ *# <metorex>/(rex|rspec)/' "$1" | sed -E \
    -e 's/^Finished in .* seconds? \(files took .* seconds? to load\)$/Finished in N seconds (files took N seconds to load)/' \
    -e 's/[0-9]+(\.[0-9]+)? seconds?/N seconds/g' \
    -e 's/0x[0-9a-f]{8,16}/0x/g' \
    -e 's/\.\.\.[0-9a-f]+>/...>/g' \
    -e 's/:[0-9]{9,}>/:ID>/g' \
    -e 's/[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}(\.[0-9]+)? [-+][0-9]{4}/TIME/g' \
    -e 's/XXXXXX\.[A-Za-z0-9]{1,10}/XXXXXX.WORK/g' \
    -e 's/metorex_rspec_suites\.[A-Za-z0-9]{6}(\\?\/)/metorex_rspec_suites.XXXXXX.WORK\1/g' \
    -e 's/metorex_rspec_suites\.[A-Za-z0-9]{1,5}\.\.\./metorex_rspec_suites.WORK.../g' \
    -e 's/, fast_paths: false//g' \
    -e 's#/([a-z]+)[0-9]+-[0-9]+/#/\1TMP/#g' \
    -e 's#/stderr[0-9]+-[0-9]+#/stderrTMP#g' \
    -e 's/@seed=[0-9]+/@seed=N/g' \
    -e 's/@load_time=[0-9.]+/@load_time=N/g'
}

differing=0
for on_log in "$COMPARE_DIR"/on/*.txt; do
  name="$(basename "$on_log")"
  off_log="$COMPARE_DIR/off/$name"
  if ! diff <(normalized "$on_log") <(normalized "$off_log") > "$COMPARE_DIR/$name.diff"; then
    differing=$((differing + 1))
    printf 'differs: %s\n' "$name"
    head -20 "$COMPARE_DIR/$name.diff"
  fi
done

total=$(find "$COMPARE_DIR/on" -name '*.txt' | wc -l | tr -d ' ')
printf '%s spec files, %s printed something different with --no-fast-paths\n' "$total" "$differing"
if [ "$differing" != "0" ]; then
  printf 'both runs are in %s\n' "$COMPARE_DIR" >&2
  exit 1
fi
rm -rf "$COMPARE_DIR"
