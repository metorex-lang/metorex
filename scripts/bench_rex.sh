#!/usr/bin/env bash
# Times Rex on the fixture suite in benchmarks/rex_suite with a release build
# and prints each number beside the one recorded in
# benchmarks/rex_suite/baseline.json.
#
#   scripts/bench_rex.sh            compare with the baseline
#   scripts/bench_rex.sh --record   write this run's numbers as the baseline
#
# Boot time is the time `require "rspec"` takes. Load time and examples per
# second come from the suite's own `Finished in` line, and peak memory is the
# maximum resident set size `/usr/bin/time` reports for the suite. Each number
# is the best of RUNS runs (3 by default). Needs MRI, which reads and writes
# the baseline.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
METOREX="$TARGET_DIR/release/metorex"
SUITE_DIR="$REPO_ROOT/benchmarks/rex_suite"
BASELINE="$SUITE_DIR/baseline.json"
RUNS="${RUNS:-3}"

cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml" >&2

case "$(uname)" in
  Darwin) TIME_FLAG=-l ;;
  *) TIME_FLAG=-v ;;
esac

# Seconds the command takes, wall clock.
seconds_for() {
  ruby -e '
    started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
    system(*ARGV, out: File::NULL, err: File::NULL) or abort "failed: #{ARGV.join(" ")}"
    print Process.clock_gettime(Process::CLOCK_MONOTONIC) - started
  ' -- "$@"
}

WORK_DIR=$(mktemp -d -t metorex_bench_rex.XXXXXX)
trap 'rm -rf "$WORK_DIR"' EXIT
results=()
for run in $(seq "$RUNS"); do
  boot=$(seconds_for "$METOREX" -e 'require "rspec"')
  (cd "$SUITE_DIR" && /usr/bin/time "$TIME_FLAG" "$METOREX" spec --no-color \
    > "$WORK_DIR/output_$run.txt" 2> "$WORK_DIR/time_$run.txt")
  results+=("$boot $WORK_DIR/output_$run.txt $WORK_DIR/time_$run.txt")
done

printf '%s\n' "${results[@]}" | ruby -rjson -e '
  record = ARGV[0] == "--record"
  baseline_path = ARGV[1]
  runs = STDIN.read.split("\n").map do |line|
    boot, output_path, time_path = line.split(" ")
    output = File.read(output_path)
    finished = output[/^Finished in ([\d.]+) seconds? \(files took ([\d.]+) seconds? to load\)$/] or
      abort "no Finished line in #{output_path}:\n#{output}"
    run_seconds = $1.to_f
    load_seconds = $2.to_f
    examples = output[/^(\d+) examples?, 0 failures/, 1] or abort "the suite failed:\n#{output}"
    time_report = File.read(time_path)
    peak = if (bytes = time_report[/(\d+)\s+maximum resident set size/, 1])
             bytes.to_i
           elsif (kilobytes = time_report[/Maximum resident set size \(kbytes\): (\d+)/, 1])
             kilobytes.to_i * 1024
           else
             abort "no peak memory in #{time_path}:\n#{time_report}"
           end
    { "boot_seconds" => boot.to_f, "load_seconds" => load_seconds,
      "examples_per_second" => examples.to_i / run_seconds, "peak_memory_bytes" => peak,
      "examples" => examples.to_i }
  end
  now = {
    "boot_seconds" => runs.map { |run| run["boot_seconds"] }.min.round(4),
    "load_seconds" => runs.map { |run| run["load_seconds"] }.min.round(4),
    "examples_per_second" => runs.map { |run| run["examples_per_second"] }.max.round(1),
    "peak_memory_bytes" => runs.map { |run| run["peak_memory_bytes"] }.min,
    "examples" => runs.first["examples"]
  }
  if record
    File.write(baseline_path, JSON.pretty_generate(now) + "\n")
    puts "wrote #{baseline_path}"
  end
  baseline = File.exist?(baseline_path) ? JSON.parse(File.read(baseline_path)) : now
  abort "the suite has #{now["examples"]} examples and the baseline #{baseline["examples"]}" if baseline["examples"] != now["examples"]
  puts format("%-20s %14s %14s %8s", "measure", "baseline", "now", "ratio")
  %w[boot_seconds load_seconds examples_per_second peak_memory_bytes].each do |key|
    puts format("%-20s %14s %14s %7.2fx", key, baseline[key], now[key], now[key].to_f / baseline[key])
  end
' -- "${1:-}" "$BASELINE"
