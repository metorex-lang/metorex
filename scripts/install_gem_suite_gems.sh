#!/usr/bin/env bash
# Installs the gems each gem suite's tests load, from the Gemfile and
# lockfile under scripts/gem_suite/bundles/<gem>, into GEM_BUNDLE, where
# scripts/run_gem_suites.sh finds them.
#
#   scripts/install_gem_suite_gems.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
export BUNDLE_PATH="${GEM_BUNDLE:-$TARGET_DIR/gem_bundle}"

for gemfile in "$REPO_ROOT"/scripts/gem_suite/bundles/*/Gemfile; do
  BUNDLE_GEMFILE="$gemfile" bundle install
done
