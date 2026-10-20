#!/usr/bin/env bash
# Installs the gems RSpec's own suites load, from scripts/rspec_suite/Gemfile
# and its lockfile, into RSPEC_BUNDLE, where scripts/run_rspec_suite.sh finds
# them. The coverage and documentation groups are left out, since no suite
# loads them.
#
#   scripts/install_rspec_suite_gems.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"

export BUNDLE_GEMFILE="$REPO_ROOT/scripts/rspec_suite/Gemfile"
export BUNDLE_PATH="${RSPEC_BUNDLE:-$TARGET_DIR/rspec_bundle}"
export BUNDLE_WITHOUT="coverage:documentation"

bundle install
