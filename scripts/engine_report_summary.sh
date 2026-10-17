#!/usr/bin/env bash
# Adds up the engine reports the processes of a run wrote into one directory,
# which is what METOREX_ENGINE_REPORT names when it names a directory, and
# prints the total in the format of one report.
#
#   scripts/engine_report_summary.sh engine-report > engine-report.txt
set -euo pipefail

DIRECTORY=${1:?usage: $0 <directory of engine reports>}

shopt -s nullglob
reports=("$DIRECTORY"/metorex-*.txt)
if [ ${#reports[@]} -eq 0 ]; then
  echo "no engine reports in $DIRECTORY" >&2
  exit 1
fi

awk '
  /^units compiled:/ { compiled += $3; next }
  /^units refused:/ { refused += $3; next }
  /^refusals:/ { next }
  NF == 2 { refusals[$2] += $1 }
  END {
    printf "units compiled: %d\n", compiled
    printf "units refused: %d\n", refused
    print "refusals:"
    for (node in refusals) printf "  %d %s\n", refusals[node], node | "sort -k1,1nr -k2,2"
  }
' "${reports[@]}"
