#!/usr/bin/env bash
# Installs the gems of each ruby-bench benchmark that loads its Gemfile, from
# the Gemfile and lockfile in its directory under benchmarks/ruby-bench, into
# the directory benchmarks/compare.rb runs it with. The lockfiles are left as
# the submodule has them.
#
#   scripts/install_benchmark_gems.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BENCHMARK_BUNDLES="${BENCHMARK_BUNDLES:-${CARGO_TARGET_DIR:-$REPO_ROOT/target}/benchmark_bundles}"

# Each bundle installs twice: once as MRI takes it, and once with the ruby
# platform variant of every gem, which Metorex takes in place of one compiled
# for MRI and builds the extensions of itself.
for gemfile in "$REPO_ROOT"/benchmarks/ruby-bench/benchmarks/*/Gemfile; do
  directory=$(dirname "$gemfile")
  grep -qs use_gemfile "$directory"/*.rb || continue
  BUNDLE_GEMFILE="$gemfile" BUNDLE_PATH="$BENCHMARK_BUNDLES" BUNDLE_FROZEN=true bundle install
  BUNDLE_GEMFILE="$gemfile" BUNDLE_PATH="$BENCHMARK_BUNDLES" BUNDLE_FROZEN=true BUNDLE_FORCE_RUBY_PLATFORM=true bundle install
done
