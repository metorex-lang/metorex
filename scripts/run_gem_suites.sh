#!/usr/bin/env bash
# Runs the test suites of the gems under gems/ against Metorex, each test
# file in a process of its own and in a copy of its gem of its own, and
# prints the totals.
#
#   scripts/run_gem_suites.sh [GEM...]
#
# GEM is a name the table below lists, the gem's directory under gems/. With none named, every suite runs.
# Each gem is a submodule at the release tag of the version Ruby 4.0.7
# ships, and its suite runs against its own library, which goes on the load
# path ahead of the one Metorex carries.
#
# ENGINE=mri runs each file under MRI in place of Metorex. With RECORD=1 the
# counts each file reached are stored as that engine's in
# scripts/gem_suite_results.tsv. A Metorex run then lists the files whose
# counts differ from MRI's there. With CHECK=1 a Metorex run is compared with
# the Metorex counts stored there instead, and fails when a file passes fewer
# examples than it did. With LOG_DIR set, what each file printed is copied
# there, named after the file. FILE_TIMEOUT is the longest one file may run,
# 300 seconds unless set, and SANDBOX_FILE_TIMEOUT the longest one of
# simplecov's sandbox spec files or RuboCop's command line spec files may,
# 900 unless set.
#
# A gem with a directory under scripts/gem_suite/bundles runs with
# BUNDLE_GEMFILE naming the Gemfile there, whose gems
# scripts/install_gem_suite_gems.sh installs under GEM_BUNDLES.

set -euo pipefail
shopt -s globstar

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
# METOREX names another binary, such as a release build.
METOREX="${METOREX:-$TARGET_DIR/debug/metorex}"
# On a Mac, one job per performance core, since a file on an efficiency core
# runs slower and can pass a spec's own time limit.
JOBS="${JOBS:-$(sysctl -n hw.perflevel0.physicalcpu 2>/dev/null || getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)}"
# The longest one test file may run before its process is stopped.
FILE_TIMEOUT="${FILE_TIMEOUT:-300}"
ENGINE="${ENGINE:-metorex}"
# The MRI the checkout names, by the path of its own binary, since each file
# runs in a directory outside the checkout, where a version manager such as
# rbenv would choose another.
MRI="${MRI:-$(cd "$REPO_ROOT" && ruby -e "print RbConfig.ruby")}"
# A test that starts `ruby` itself reaches the same one.
if [ "$ENGINE" = "mri" ]; then
  export PATH="$(dirname "$MRI"):$PATH"
fi
case "$ENGINE" in
  metorex | mri) ;;
  *) echo "run_gem_suites.sh: ENGINE is metorex or mri" >&2; exit 2 ;;
esac
export SUITE_RESULTS="$REPO_ROOT/scripts/gem_suite_results.tsv"
BUNDLES="$REPO_ROOT/scripts/gem_suite/bundles"
# shellcheck source=gem_suite/bundle_paths.sh
. "$REPO_ROOT/scripts/gem_suite/bundle_paths.sh"

if [ ! -f "$METOREX" ]; then
  echo "Building metorex..."
  cargo build --manifest-path "$REPO_ROOT/Cargo.toml"
fi

# Each gem's test files, as globs under its directory, and the arguments
# each file runs with: its library and test helpers on the load path, and
# what its own runner requires ahead of the files. A fourth field names a
# script run in the copy of the gem before the file, which writes what the
# gem's own build would generate. HELPERS holds the
# assertions Ruby's own repository keeps in tool/lib, which
# scripts/vendor_gems.rb copies in.
HELPERS="$REPO_ROOT/scripts/gem_suite/lib"
# The rspec command, for the suites written as RSpec specs.
RSPEC="$REPO_ROOT/scripts/gem_suite/rspec.rb"
SUITES=(
  "minitest|test/minitest/test_*.rb|-I lib -I test"
  "test-unit|test/test-*.rb test/*/test-*.rb|-I lib -I test -r test/unit"
  "power_assert|test/*_test.rb|-w -I lib -I test -r ./test/test_helper.rb"
  "rake|test/test_*.rb|-I lib -I test"
  "rexml|test/test_*.rb test/*/test_*.rb test/*/*/test_*.rb|-I lib -I test -I $HELPERS -r helper"
  "rss|test/test-*.rb|-I lib -I test -r test-unit -r rss-testcase"
  "net-imap|test/net/imap/test_*.rb|-I lib -I test/lib -I $HELPERS -r helper"
  "net-pop|test/net/pop/test_*.rb|-I lib -I test/lib -r helper"
  "net-smtp|test/net/smtp/test_*.rb|-I lib -I test"
  "base64|test/base64/test_*.rb|-I lib -I test"
  "json|test/json/*_test.rb|-I lib -I $HELPERS"
  "bigdecimal|test/bigdecimal/test_*.rb|-w -I lib -I test/lib -I $HELPERS -r helper"
  "rack|test/spec_*.rb|-I lib -I test -r bundler/setup"
  "rack-test|spec/rack/*_spec.rb spec/rack/test/*_spec.rb|-I lib -I spec -r bundler/setup"
  "sinatra/rack-protection|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "faraday|spec/faraday_spec.rb spec/faraday/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "faraday-net_http|spec/**/*_spec.rb|-I lib -I spec -I $REPO_ROOT/gems/faraday/spec/external_adapters -r bundler/setup $RSPEC"
  "multipart-post|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "excon|test/*_test.rb|-I lib -I test -r bundler/setup"
  "httpclient|test/test_*.rb|-I lib -I test -r bundler/setup"
  "http-cookie|test/test_*.rb|-I lib -I test -r bundler/setup"
  "domain_name|test/test_*.rb|-I lib -I test"
  "connection_pool|test/test_*.rb|-I lib -I test"
  "websocket-extensions|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "i18n|test/**/*_test.rb|-w -I lib -I test -r bundler/setup"
  "tzinfo|test/ts_all_ruby_format1.rb test/ts_all_ruby_format2.rb test/ts_all_zoneinfo.rb|-w -I lib -r bundler/setup"
  "multi_json|test/**/*_test.rb|-I lib -I test"
  "multi_xml|test/**/*_test.rb|-I lib -I test -r bundler/setup"
  "builder|test/test*.rb|-I lib -I . -I test -r bundler/setup"
  "mime-types|test/test_*.rb|-w -I lib -I test -I . -r bundler/setup -r minitest/autorun"
  "mini_mime|test/**/*_test.rb|-I lib -I test -r bundler/setup"
  "mail|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "addressable|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "public_suffix|test/**/*_test.rb|-I lib -I test -r bundler/setup"
  "jwt|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "dotenv|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "dotenv|test/**/*_test.rb|-I lib -I test -r bundler/setup"
  "netrc|test/test_*.rb|-I lib -I test"
  "rubyzip|test/**/*_test.rb|-I lib -I test -r bundler/setup"
  "thor|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "rainbow|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "ruby-progressbar|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "unicode-display_width|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "parallel|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "method_source|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "coderay|test/functional/suite.rb test/unit/suite.rb|-I lib -r bundler/setup"
  "coderay|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "pry|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "simplecov|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "docile|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "mini_portile2|test/test_*.rb|-I lib -I test -r bundler/setup"
  "ast|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "parser|test/**/test_*.rb|-w -I lib -I test -r bundler/setup|$REPO_ROOT/scripts/gem_suite/released_generated_files.rb"
  "regexp_parser|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC|$REPO_ROOT/scripts/gem_suite/released_generated_files.rb"
  "rubocop-ast|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC|$REPO_ROOT/scripts/gem_suite/released_generated_files.rb"
  "rubocop|spec/**/*_spec.rb|-I lib -I spec -r bundler/setup $RSPEC"
  "racc|test/test_*.rb|--enable-frozen-string-literal -I lib -I test/lib -I $HELPERS -r helper|$REPO_ROOT/scripts/gem_suite/racc_parser_text.rb"
)

# The gems whose test files run one after another, since each of them
# starts a server on the same fixed port.
ONE_AT_A_TIME=(excon mini_portile2)

# The longest one of simplecov's sandbox spec files may run, each of which
# boots a Rails application and runs its tests once per example, or one of
# RuboCop's command line spec files, each of which runs RuboCop over a
# project once per example.
SANDBOX_FILE_TIMEOUT="${SANDBOX_FILE_TIMEOUT:-900}"
one_at_a_time() {
  local held
  for held in "${ONE_AT_A_TIME[@]}"; do
    [ "$held" = "$1" ] && return 0
  done
  return 1
}

WANTED=("$@")
wanted() {
  [ ${#WANTED[@]} -eq 0 ] && return 0
  local held
  for held in "${WANTED[@]}"; do
    [ "$held" = "$1" ] && return 0
  done
  return 1
}

WORK_DIR=$(mktemp -d -t metorex_gem_suites.XXXXXX)
trap 'rm -rf "$WORK_DIR"' EXIT
# A test that starts `ruby` by name, as thor's do, starts Metorex in a
# Metorex run, through a link named `ruby` ahead of the others on PATH.
if [ "$ENGINE" = "metorex" ]; then
  mkdir -p "$WORK_DIR/bin"
  ln -s "$(cd "$(dirname "$METOREX")" && pwd -P)/$(basename "$METOREX")" "$WORK_DIR/bin/ruby"
  # A test that runs `bundle`, as simplecov's sandboxes do, runs the
  # Bundler Metorex carries, as `bundle` runs MRI's in an MRI run.
  cat > "$WORK_DIR/bin/bundle" <<'BUNDLE'
#!/bin/sh
exec "$(dirname "$0")/ruby" -e 'require "bundler"; require "bundler/friendly_errors"; Bundler.with_friendly_errors { require "bundler/cli"; Bundler::CLI.start(ARGV, debug: true) }' -- "$@"
BUNDLE
  chmod +x "$WORK_DIR/bin/bundle"
  export PATH="$WORK_DIR/bin:$PATH"
fi
QUEUE="$WORK_DIR/queue.tsv"
: > "$QUEUE"
for row in "${SUITES[@]}"; do
  IFS='|' read -r gem patterns arguments prepare <<< "$row"
  wanted "$gem" || continue
  if [ ! -d "$REPO_ROOT/gems/$gem/lib" ]; then
    echo "gems/$gem is missing, run: git submodule update --init gems/$gem" >&2
    exit 2
  fi
  if [ -f "$BUNDLES/${gem##*/}/Gemfile" ] && ! BUNDLE_GEMFILE="$BUNDLES/${gem##*/}/Gemfile" BUNDLE_PATH="$(bundle_path_of "${gem##*/}")" bundle check > /dev/null 2>&1; then
    echo "the gems $gem's tests load are missing, run: scripts/install_gem_suite_gems.sh" >&2
    exit 2
  fi
  # Read without expanding, since the checkout's own spec/ would match.
  read -r -a pattern_list <<< "$patterns"
  files=$(for pattern in "${pattern_list[@]}"; do
    (cd "$REPO_ROOT/gems/$gem" && compgen -G "$pattern" || true) | sort
  done)
  if one_at_a_time "$gem"; then
    printf '%s\t%s\t%s\t%s\n' "$gem" "$(echo $files)" "$arguments" "$prepare" >> "$QUEUE"
  else
    for file in $files; do
      printf '%s\t%s\t%s\t%s\n' "$gem" "$file" "$arguments" "$prepare" >> "$QUEUE"
    done
  fi
done

# Runs one test file in a copy of its gem, writing what it printed next to
# it, and prints the file's name and its summary line.
run_one() {
  local gem="$1" file="$2" arguments="$3" prepare="$4"
  local name="$gem/$file"
  local log="$WORK_DIR/logs/${name//\//__}.txt"
  local run_dir="$WORK_DIR/runs/${name//\//__}/$gem"
  mkdir -p "$WORK_DIR/logs" "$run_dir"
  (cd "$REPO_ROOT/gems/$gem" && tar --exclude .git -cf - .) | (cd "$run_dir" && tar -xf -)
  local limit="$FILE_TIMEOUT"
  case "$name" in
    simplecov/spec/sandbox/* | rubocop/spec/rubocop/cli/* | rubocop/spec/rubocop/cli_spec.rb | rubocop/spec/rubocop/server/*) limit="$SANDBOX_FILE_TIMEOUT" ;;
  esac
  local command
  if [ "$ENGINE" = "mri" ]; then
    command=("$MRI")
  else
    command=("$METOREX")
  fi
  (
    cd "$run_dir"
    if [ -f "$BUNDLES/${gem##*/}/Gemfile" ]; then
      export BUNDLE_GEMFILE="$BUNDLES/${gem##*/}/Gemfile" BUNDLE_PATH="$(bundle_path_of "${gem##*/}")"
    fi
    if [ "$prepare" != "$NO_PREPARE" ]; then
      "${command[@]}" "$prepare" > "$log" 2>&1 < /dev/null
    fi
    # The file runs in a process group of its own, and the whole group is
    # stopped when it ends or runs out of time, so a process a test started
    # and left behind does not keep running into the next files.
    # shellcheck disable=SC2086
    perl -e '
      my $limit = shift;
      my $pid = fork;
      if ($pid == 0) { setpgrp(0, 0); exec @ARGV or exit 127 }
      $SIG{ALRM} = sub { kill "KILL", -$pid; waitpid($pid, 0); exit 142 };
      alarm $limit;
      waitpid($pid, 0);
      my $status = $?;
      kill "KILL", -$pid;
      exit($status >> 8);
    ' "$limit" "${command[@]}" $arguments "$file" >> "$log" 2>&1 < /dev/null
  ) || true
  local summary
  summary=$(sed 's/\x1b\[[0-9;]*m//g' "$log" | grep -E '^[0-9]+ (tests|runs|examples?), ' | tail -1 || true)
  printf '%s\t%s\n' "$name" "${summary:-did not finish}"
}
# Runs each of the space-separated files a queue entry names, in turn.
run_files() {
  local gem="$1" files="$2" arguments="$3" prepare="$4" file
  for file in $files; do
    run_one "$gem" "$file" "$arguments" "$prepare"
  done
}
export -f run_one run_files
NO_PREPARE=none
export -f bundle_path_of
export MRI WORK_DIR REPO_ROOT BUNDLES GEM_BUNDLES METOREX FILE_TIMEOUT SANDBOX_FILE_TIMEOUT ENGINE NO_PREPARE
# net-imap's helper measures coverage with SimpleCov unless told not to.
export SIMPLECOV_DISABLE=1
# A test that omits itself under CI would count differently there than in
# the recorded runs.
unset CI
# Minitest runs a file's tests in an order drawn from this seed, so both
# engines run them in the same order and a test that depends on another
# counts the same under each.
export SEED="${SEED:-1}"

RESULTS="$WORK_DIR/results.tsv"
while IFS=$'\t' read -r gem file arguments prepare; do
  # BSD xargs drops an empty argument, so a row with no prepare script
  # passes NO_PREPARE in its place.
  printf '%s\0%s\0%s\0%s\0' "$gem" "$file" "$arguments" "${prepare:-$NO_PREPARE}"
done < "$QUEUE" | xargs -0 -n 4 -P "$JOBS" bash -c 'run_files "$@"' _ > "$RESULTS"
if [ -n "${LOG_DIR:-}" ]; then
  mkdir -p "$LOG_DIR"
  cp "$WORK_DIR"/logs/*.txt "$LOG_DIR/"
fi
if [ -n "${RECORD:-}" ]; then
  ruby "$REPO_ROOT/scripts/suite_results.rb" record "$ENGINE" "$RESULTS"
fi
checked=0
if [ "$ENGINE" = "metorex" ] && [ -n "${CHECK:-}" ]; then
  echo "Files whose counts differ from the recorded ones:"
  ruby "$REPO_ROOT/scripts/suite_results.rb" regressions "$RESULTS" || checked=1
elif [ "$ENGINE" = "metorex" ]; then
  echo "Files whose counts differ from MRI's:"
  ruby "$REPO_ROOT/scripts/suite_results.rb" differences "$RESULTS" || true
fi

files=0
unfinished=0
while IFS=$'\t' read -r name summary; do
  files=$((files + 1))
  if [ "$summary" = "did not finish" ]; then
    unfinished=$((unfinished + 1))
    printf 'did not finish: %s\n' "$name"
  fi
done < <(sort "$RESULTS")
printf '%s files, %s did not finish\n' "$files" "$unfinished"
exit "$checked"
