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
  /^units on bytecode:/ { bytecode += $4; next }
  /^units on the tree walker:/ { tree += $6; next }
  /^fallbacks:/ { next }
  NF == 2 { fallbacks[$2] += $1 }
  END {
    printf "units on bytecode: %d\n", bytecode
    printf "units on the tree walker: %d\n", tree
    print "fallbacks:"
    for (node in fallbacks) printf "  %d %s\n", fallbacks[node], node | "sort -k1,1nr -k2,2"
  }
' "${reports[@]}"
