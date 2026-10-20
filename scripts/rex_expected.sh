#!/usr/bin/env bash
# Writes the output RSpec 3.13 gives for a Rex fixture, which the Rex tests
# compare `metorex spec` against.
#
#   scripts/rex_expected.sh FIXTURE NAME [ARGUMENTS...]
#
# runs the rspec executable with ARGUMENTS in tests/_examples/rex/FIXTURE and
# writes tests/_examples/rex/FIXTURE/expected_NAME.txt: stdout and stderr as
# one stream, then a last line `exit STATUS`.
#
# What changes from run to run is written as a placeholder: the timing line
# as `Finished in N seconds (files took N seconds to load)`, a duration
# `--bisect` prints in parentheses as `(N seconds)` and the one it ends with
# as `in N seconds.`, an object address such as 0x0000000100cfac28 as 0x,
# and in a JSON document each duration and run time as N and the version
# as V, each `--profile` time and percentage as N, each HTML duration as
# N, and the checkout's directory, which a deprecation's call site names,
# as ROOT.
#
# SPEC_OPTS set in this environment is passed on. SHELL is set to /bin/zsh,
# since RSpec quotes an example id for zsh and not for bash.
#
# RSpec runs with scripts/rspec_without_coderay on its load path, so a
# failing line prints without syntax highlighting, as Rex prints it.
#
# Needs MRI with the rspec-core, rspec-expectations, and rspec-mocks gems.

set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "usage: $0 FIXTURE NAME [ARGUMENTS...]" >&2
  exit 2
fi

fixture="$1"
name="$2"
shift 2

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
directory="$root/tests/_examples/rex/$fixture"
core="$(ruby -e 'print Gem::Specification.find_by_name("rspec-core").gem_dir')"
rspec="$core/exe/rspec"

cd "$directory"
set +e
output="$(SHELL=/bin/zsh ruby -I "$root/scripts/rspec_without_coderay" "$rspec" "$@" 2>&1)"
status=$?
set -e

printf '%s\n' "$output" \
  | sed -E 's/^Finished in .* seconds \(files took .* seconds to load\)$/Finished in N seconds (files took N seconds to load)/' \
  | sed -E 's/\([0-9.]+ seconds?\)/(N seconds)/g' \
  | sed -E 's/ in [0-9.]+ seconds?\.$/ in N seconds./' \
  | sed -E 's/0x[0-9a-f]{8,16}/0x/g' \
  | sed -E 's/"(duration|run_time|slowest|total|average|total_time)":[-0-9.e]+/"\1":N/g' \
  | sed -E 's/"version":"[^"]*"/"version":"V"/' \
  | sed -E "s/<span class='duration'>[0-9.]+s<\/span>/<span class='duration'>Ns<\/span>/; s/<span class=\"duration\">[0-9.]+s<\/span>/<span class=\"duration\">Ns<\/span>/; s/<strong>[0-9.]+ seconds<\/strong>/<strong>N seconds<\/strong>/" \
  | sed -E 's/^(Top [0-9]+ slowest examples) \([0-9.]+ seconds, [0-9.]+% of total time\):$/\1 (N seconds, N% of total time):/' \
  | sed -E 's/^    [0-9.]+ seconds (\.\/.*)$/    N seconds \1/' \
  | sed -E 's/^    [0-9.]+ seconds average \([0-9.]+ seconds \/ (.*)\) (\.\/.*)$/    N seconds average (N seconds \/ \1) \2/' \
  | ROOT="$root" ruby -pe 'gsub(ENV["ROOT"], "ROOT")' \
  > "expected_$name.txt"
printf 'exit %s\n' "$status" >> "expected_$name.txt"
echo "wrote tests/_examples/rex/$fixture/expected_$name.txt (exit $status)"
