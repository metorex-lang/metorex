#!/usr/bin/env bash
# Profiles one metorex run with the operating system's sampling profiler and
# writes the report to a file. The binary is the `profiling` build, a release
# build with line tables, so the report names the file and line of each
# frame.
#
#   scripts/profile.sh REPORT [metorex arguments...]
#
# On macOS the profiler is `sample`, which comes with the system, and the
# report is its call tree followed by the functions most often on top of the
# stack. On Linux it is `perf`, and the report is `perf report --stdio`, with
# the recording beside it as REPORT.data.

set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "usage: scripts/profile.sh REPORT [metorex arguments...]" >&2
  exit 2
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
REPORT="$1"
shift

cargo build --profile profiling --manifest-path "$REPO_ROOT/Cargo.toml" >&2
METOREX="$TARGET_DIR/profiling/metorex"

case "$(uname)" in
  Darwin)
    "$METOREX" "$@" &
    program=$!
    # `sample` stops when the program exits, so the hour is an upper bound
    # rather than how long it samples.
    sample "$program" 3600 -mayDie -file "$REPORT" > /dev/null 2>&1 &
    profiler=$!
    status=0
    wait "$program" || status=$?
    wait "$profiler" || true
    ;;
  *)
    status=0
    perf record --call-graph dwarf -o "$REPORT.data" -- "$METOREX" "$@" || status=$?
    perf report -i "$REPORT.data" --stdio --no-children > "$REPORT"
    ;;
esac
echo "wrote $REPORT" >&2
exit "$status"
