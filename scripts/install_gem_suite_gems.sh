#!/usr/bin/env bash
# Installs the gems each gem suite's tests load, from the Gemfile and
# lockfile under scripts/gem_suite/bundles/<gem>, into the directory
# scripts/gem_suite/bundle_paths.sh names for the gem, where
# scripts/run_gem_suites.sh finds them.
#
#   scripts/install_gem_suite_gems.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=gem_suite/bundle_paths.sh
. "$REPO_ROOT/scripts/gem_suite/bundle_paths.sh"

for gemfile in "$REPO_ROOT"/scripts/gem_suite/bundles/*/Gemfile; do
  gem=$(basename "$(dirname "$gemfile")")
  BUNDLE_GEMFILE="$gemfile" BUNDLE_PATH="$(bundle_path_of "$gem")" bundle install
done

# simplecov's sandbox specs run their projects with simplecov's own Gemfile
# and lockfile, which are left as the submodule has them.
BUNDLE_GEMFILE="$REPO_ROOT/gems/simplecov/Gemfile" BUNDLE_PATH="$(bundle_path_of simplecov)" \
  BUNDLE_FROZEN=true bundle install
