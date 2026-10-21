#!/usr/bin/env bash
# Runs the test suites of the gems under gems/ against Metorex, each test
# file in a process of its own and in a copy of its gem of its own, and
# prints the totals.
#
#   scripts/run_gem_suites.sh [GEM...]
#
# GEM is a name the table below lists. With none named, every suite runs.
# Each gem is a submodule at the release tag of the version Ruby 4.0.7
# ships, and its suite runs against its own library, which goes on the load
# path ahead of the one Metorex carries.
#
# ENGINE=mri runs each file under MRI in place of Metorex. With RECORD=1 the
# counts each file reached are stored as that engine's in
# scripts/gem_suite_results.tsv. A Metorex run then lists the files whose
# counts differ from MRI's there. With LOG_DIR set, what each file printed is
# copied there, named after the file.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
# METOREX names another binary, such as a release build.
METOREX="${METOREX:-$TARGET_DIR/debug/metorex}"
JOBS="${JOBS:-$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)}"
# The longest one test file may run before its process is stopped.
FILE_TIMEOUT="${FILE_TIMEOUT:-300}"
ENGINE="${ENGINE:-metorex}"
case "$ENGINE" in
  metorex | mri) ;;
  *) echo "run_gem_suites.sh: ENGINE is metorex or mri" >&2; exit 2 ;;
esac
export SUITE_RESULTS="$REPO_ROOT/scripts/gem_suite_results.tsv"

if [ ! -f "$METOREX" ]; then
  echo "Building metorex..."
  cargo build --manifest-path "$REPO_ROOT/Cargo.toml"
fi

# Each gem's test files, as globs under its directory, and the arguments
# each file runs with: its library and test helpers on the load path, and
# what its own runner requires ahead of the files. HELPERS holds the
# assertions Ruby's own repository keeps in tool/lib, which
# scripts/vendor_gems.rb copies in.
HELPERS="$REPO_ROOT/scripts/gem_suite/lib"
SUITES=(
  "minitest|test/minitest/test_*.rb|-I lib -I test"
  "test-unit|test/test-*.rb test/*/test-*.rb|-I lib -I test -r test/unit"
  "power_assert|test/*_test.rb|-w -I lib -I test -r ./test/test_helper.rb"
  "rake|test/test_*.rb|-I lib -I test"
  "rexml|test/test_*.rb test/*/test_*.rb test/*/*/test_*.rb|-I lib -I test -I $HELPERS -r helper"
  "rss|test/test-*.rb|-I lib -I test -r test-unit -r rss-testcase"
  "net-imap|test/net/imap/test_*.rb|-I lib -I test/lib -I $HELPERS -r helper"
  "net-pop|test/net/pop/test_*.rb|-I lib -I test/lib -r helper"
  "net-smtp|test/net/smtp/test_*.rb|-I lib -I test"
)

WANTED=("$@")
wanted() {
  [ ${#WANTED[@]} -eq 0 ] && return 0
  local held
  for held in "${WANTED[@]}"; do
    [ "$held" = "$1" ] && return 0
  done
  return 1
}

WORK_DIR=$(mktemp -d -t metorex_gem_suites.XXXXXX)
trap 'rm -rf "$WORK_DIR"' EXIT
QUEUE="$WORK_DIR/queue.tsv"
: > "$QUEUE"
for row in "${SUITES[@]}"; do
  IFS='|' read -r gem patterns arguments <<< "$row"
  wanted "$gem" || continue
  if [ ! -d "$REPO_ROOT/gems/$gem/lib" ]; then
    echo "gems/$gem is missing, run: git submodule update --init gems/$gem" >&2
    exit 2
  fi
  for pattern in $patterns; do
    (cd "$REPO_ROOT/gems/$gem" && compgen -G "$pattern" || true) | sort | while read -r file; do
      printf '%s\t%s\t%s\n' "$gem" "$file" "$arguments" >> "$QUEUE"
    done
  done
done

# Runs one test file in a copy of its gem, writing what it printed next to
# it, and prints the file's name and its summary line.
run_one() {
  local gem="$1" file="$2" arguments="$3"
  local name="$gem/$file"
  local log="$WORK_DIR/logs/${name//\//__}.txt"
  local run_dir="$WORK_DIR/runs/${name//\//__}/$gem"
  mkdir -p "$WORK_DIR/logs" "$run_dir"
  (cd "$REPO_ROOT/gems/$gem" && tar --exclude .git -cf - .) | (cd "$run_dir" && tar -xf -)
  local command
  if [ "$ENGINE" = "mri" ]; then
    command=(ruby)
  else
    command=("$METOREX")
  fi
  (
    cd "$run_dir"
    # shellcheck disable=SC2086
    perl -e 'alarm shift; exec @ARGV' "$FILE_TIMEOUT" "${command[@]}" $arguments "$file" > "$log" 2>&1 < /dev/null
  ) || true
  local summary
  summary=$(sed 's/\x1b\[[0-9;]*m//g' "$log" | grep -E '^[0-9]+ (tests|runs), ' | tail -1 || true)
  printf '%s\t%s\n' "$name" "${summary:-did not finish}"
}
export -f run_one
export WORK_DIR REPO_ROOT METOREX FILE_TIMEOUT ENGINE
# net-imap's helper measures coverage with SimpleCov unless told not to.
export SIMPLECOV_DISABLE=1

RESULTS="$WORK_DIR/results.tsv"
while IFS=$'\t' read -r gem file arguments; do
  printf '%s\0%s\0%s\0' "$gem" "$file" "$arguments"
done < "$QUEUE" | xargs -0 -n 3 -P "$JOBS" bash -c 'run_one "$@"' _ > "$RESULTS"
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

files=0
unfinished=0
while IFS=$'\t' read -r name summary; do
  files=$((files + 1))
  if [ "$summary" = "did not finish" ]; then
    unfinished=$((unfinished + 1))
    printf 'did not finish: %s\n' "$name"
  fi
done < <(sort "$RESULTS")
printf '%s files, %s did not finish\n' "$files" "$unfinished"
