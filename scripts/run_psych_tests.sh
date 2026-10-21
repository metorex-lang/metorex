#!/bin/bash
# Runs each file of Psych's own test suite, from the Ruby release unpacked
# under target/ruby-release, under MRI and under Metorex, and prints the
# files whose summaries differ. METOREX names the binary, the release build
# by default, and LOG_DIR keeps each run's output.
#
#   scripts/run_psych_tests.sh [test_file.rb ...]

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
METOREX="${METOREX:-$REPO_ROOT/target/release/metorex}"
TEST_DIR=$(ls -d "$REPO_ROOT"/target/ruby-release/ruby-*/test/psych 2>/dev/null | tail -1)
HELPERS="$REPO_ROOT/scripts/gem_suite/lib"
LOG_DIR="${LOG_DIR:-$(mktemp -d)}"
FILE_TIMEOUT="${FILE_TIMEOUT:-120}"

if [ -z "$TEST_DIR" ]; then
  echo "run_psych_tests.sh: no test/psych under target/ruby-release; unpack the release first" >&2
  exit 2
fi

if [ "$#" -eq 0 ]; then
  set -- "$TEST_DIR"/test_*.rb "$TEST_DIR"/*/test_*.rb
fi

differing=0
for file in "$@"; do
  relative="${file#"$TEST_DIR"/}"
  name="${relative%.rb}"
  name="${name//\//__}"
  (cd "$TEST_DIR" && perl -e 'alarm shift; exec @ARGV' "$FILE_TIMEOUT" ruby -I . -I "$HELPERS" "$relative") > "$LOG_DIR/$name.mri" 2>&1
  (cd "$TEST_DIR" && perl -e 'alarm shift; exec @ARGV' "$FILE_TIMEOUT" "$METOREX" -I . -I "$HELPERS" "$relative") > "$LOG_DIR/$name.metorex" 2>&1
  mri=$(grep -E "^[0-9]+ tests" "$LOG_DIR/$name.mri" | tail -1)
  metorex=$(grep -E "^[0-9]+ tests" "$LOG_DIR/$name.metorex" | tail -1)
  if [ "$mri" != "$metorex" ]; then
    echo "$name: MRI [$mri] Metorex [${metorex:-did not finish}]"
    differing=$((differing + 1))
  fi
done
echo "$# files, $differing differ from MRI; output is in $LOG_DIR"
[ "$differing" -eq 0 ]
