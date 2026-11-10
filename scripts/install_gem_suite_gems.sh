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

# Each bundle installs twice: once as MRI takes it, and once with the ruby
# platform variant of every gem, which Metorex takes in place of one compiled
# for MRI and builds the extensions of itself.
for gemfile in "$REPO_ROOT"/scripts/gem_suite/bundles/*/Gemfile; do
  gem=$(basename "$(dirname "$gemfile")")
  BUNDLE_GEMFILE="$gemfile" BUNDLE_PATH="$(bundle_path_of "$gem")" bundle install
  BUNDLE_GEMFILE="$gemfile" BUNDLE_PATH="$(bundle_path_of "$gem")" BUNDLE_FORCE_RUBY_PLATFORM=true bundle install
done

# simplecov's sandbox specs run their projects with simplecov's own Gemfile
# and lockfile, which are left as the submodule has them.
BUNDLE_GEMFILE="$REPO_ROOT/gems/simplecov/Gemfile" BUNDLE_PATH="$(bundle_path_of simplecov)" \
  BUNDLE_FROZEN=true bundle install

# Metorex installs gems of its own into these directories as simplecov's Rails
# sandboxes run. MRI builds its own extensions for them, and every executable
# is written to start with `#!/usr/bin/env ruby`, as Bundler writes them.
ruby_version=$(ruby -e 'print RbConfig::CONFIG["ruby_version"]')
for bundle_path in "$(bundle_path_of shared)" "$(bundle_path_of simplecov)"; do
  GEM_HOME="$bundle_path/ruby/$ruby_version" GEM_PATH="$bundle_path/ruby/$ruby_version" \
    ruby "$REPO_ROOT/scripts/gem_suite/prepare_bundle_for_mri.rb" "$bundle_path/ruby/$ruby_version"
done
