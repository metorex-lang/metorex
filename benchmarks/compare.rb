# Times Metorex, with its JIT and with METOREX_JIT=0, beside CRuby with and
# without YJIT and prints the results as a Markdown table. Run it from the repository root with a release build:
#
#   cargo build --release
#   ruby benchmarks/compare.rb
#
# The rows are:
#
# - `empty`, an empty script, which times starting up
# - the scripts in benchmarks/workloads and benchmarks/memory, whose output
#   must match across engines
# - every benchmark in benchmarks/ruby-bench (github.com/ruby/ruby-bench) in its
#   headline, other, and micro categories that needs no gems
# - the Rex suite in benchmarks/rex_suite, run with `metorex spec` and with
#   MRI's `rspec`
#
# Startup, workload, and Rex suite times are the fastest of three whole runs.
# A ruby-bench time is the mean of one iteration after the warmup ones, run
# under ruby-bench's own harness with the settings in HARNESS_SETTINGS. Peak memory is the largest resident set
# size of the run.
#
# Options:
#
#   --metorex=PATH   another metorex binary
#   --only=a,b       only the rows with these names
#   --without-no-jit no METOREX_JIT=0 column, for a binary without a JIT

require "json"
require "open3"
require "rbconfig"
require "tmpdir"
require "yaml"

options = ARGV.to_h do |argument|
  name, value = argument.delete_prefix("--").split("=", 2)
  [name, value]
end
METOREX = options.fetch("metorex", File.expand_path("../target/release/metorex", __dir__))
abort "no metorex binary at #{METOREX}" unless File.executable?(METOREX)
ONLY = options["only"]&.split(",")

RUBY_BENCH = File.join(__dir__, "ruby-bench")
abort "benchmarks/ruby-bench is missing, run: git submodule update --init" unless
  File.exist?(File.join(RUBY_BENCH, "benchmarks.yml"))

ENGINES = {
  "CRuby" => [RbConfig.ruby],
  "CRuby --yjit" => [RbConfig.ruby, "--yjit"],
  "Metorex" => [METOREX],
  "Metorex, no JIT" => [METOREX]
}.reject { |engine, _| engine == "Metorex, no JIT" && options.key?("without-no-jit") }.freeze
NO_JIT = ENGINES.key?("Metorex, no JIT")

# What each engine runs with besides its command.
ENVIRONMENTS = Hash.new({}).merge("Metorex, no JIT" => { "METOREX_JIT" => "0" }).freeze

# Iterations under ruby-bench's harness. CRuby runs the harness's defaults,
# as ruby-bench's own runner does: 15 warmup iterations, then at least 10 more
# and at least 10 seconds of them, which is long enough for YJIT to compile
# the benchmark. Metorex runs one warmup iteration, for the caches its
# strings and patterns build on first use and the units its JIT compiles
# once they have run 30 times, and two more.
HARNESS_SETTINGS = {
  "CRuby" => {},
  "CRuby --yjit" => {},
  "Metorex" => { "WARMUP_ITRS" => "1", "MIN_BENCH_ITRS" => "2", "MIN_BENCH_TIME" => "0" },
  "Metorex, no JIT" => { "WARMUP_ITRS" => "1", "MIN_BENCH_ITRS" => "2", "MIN_BENCH_TIME" => "0" }
}.freeze

# The flag that makes /usr/bin/time report the peak resident set size, and the
# line and unit it reports it in, which differ between macOS and Linux.
PEAK_MEMORY = if RUBY_PLATFORM.include?("darwin")
  { flag: "-l", pattern: /(\d+)\s+maximum resident set size/, bytes_per_unit: 1 }
else
  { flag: "-v", pattern: /Maximum resident set size \(kbytes\): (\d+)/, bytes_per_unit: 1024 }
end

Row = Struct.new(:name, :seconds, :peaks)

# One run of `command` in `directory`, answering the seconds it took, its
# peak resident set size in bytes, and what it printed.
def timed(command, directory: Dir.pwd, environment: {})
  started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  printed, reported, status = Open3.capture3(environment, "/usr/bin/time", PEAK_MEMORY[:flag], *command,
                                             chdir: directory)
  finished = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  abort "#{command.join(" ")} failed:\n#{printed}#{reported}" unless status.success?
  peak = reported[PEAK_MEMORY[:pattern], 1] or abort "no peak memory in:\n#{reported}"
  [finished - started, peak.to_i * PEAK_MEMORY[:bytes_per_unit], printed]
end

# The fastest of three runs and the largest peak memory of the three, with
# what the first run printed. With `same_output`, every run must print the
# same thing.
def fastest(command, same_output: true, **options)
  runs = Array.new(3) { timed(command, **options) }
  outputs = runs.map(&:last).uniq
  abort "#{command.join(" ")} printed different things" if same_output && outputs.size > 1
  [runs.map(&:first).min, runs.map { |run| run[1] }.max, outputs.first]
end

def script_row(script)
  name = File.basename(script, ".rb")
  results = ENGINES.to_h do |engine, command|
    [engine, fastest(command + [script], environment: ENVIRONMENTS[engine])]
  end
  outputs = results.values.map(&:last).uniq
  abort "#{name} printed different things under different engines" unless outputs.size == 1
  Row.new(name, results.transform_values(&:first), results.transform_values { |result| result[1] })
end

# The ruby-bench benchmarks in the categories its runner runs by default,
# each as its name, its file, and whether it needs gems. A benchmark is a file
# of its own, a directory with a benchmark.rb, or one file of a directory
# that holds several, named after the directory and the file.
def ruby_bench_benchmarks
  YAML.load_file(File.join(RUBY_BENCH, "benchmarks.yml")).filter_map do |name, settings|
    next unless %w[headline other micro].include?(settings.fetch("category", "other"))
    next if settings["ractor_only"]
    single_file = File.join(RUBY_BENCH, "benchmarks", "#{name}.rb")
    next [name, single_file, false] if File.exist?(single_file)
    directory = File.join(RUBY_BENCH, "benchmarks", name)
    file = File.join(directory, "benchmark.rb")
    unless File.directory?(directory)
      parts = name.split("-")
      taken = (parts.size - 1).downto(1).find do |count|
        File.directory?(File.join(RUBY_BENCH, "benchmarks", parts.first(count).join("-")))
      end or next
      directory = File.join(RUBY_BENCH, "benchmarks", parts.first(taken).join("-"))
      file = File.join(directory, "#{parts.drop(taken).join("-")}.rb")
    end
    [name, file, File.exist?(File.join(directory, "Gemfile"))]
  end.sort
end

def ruby_bench_row(name, benchmark)
  seconds = {}
  peaks = {}
  ENGINES.each do |engine, command|
    Dir.mktmpdir("metorex-compare") do |scratch|
      results = File.join(scratch, "results.json")
      environment = HARNESS_SETTINGS[engine].merge(ENVIRONMENTS[engine], "RESULT_JSON_PATH" => results)
      harness = File.join(RUBY_BENCH, "harness")
      timed(command + ["-I", harness, benchmark], directory: RUBY_BENCH, environment: environment)
      recorded = JSON.parse(File.read(results))
      seconds[engine] = recorded["bench"].sum / recorded["bench"].size
      peaks[engine] = recorded["maxrss"]
    end
  end
  Row.new(name, seconds, peaks)
end

def rex_suite_row
  suite = File.join(__dir__, "rex_suite")
  commands = {
    "CRuby" => [RbConfig.ruby, "-S", "rspec", "--no-color"],
    "CRuby --yjit" => [RbConfig.ruby, "--yjit", "-S", "rspec", "--no-color"],
    "Metorex" => [METOREX, "spec", "--no-color"],
    "Metorex, no JIT" => [METOREX, "spec", "--no-color"]
  }.slice(*ENGINES.keys)
  results = commands.to_h do |engine, command|
    # The suite's report includes how long it took, so runs differ.
    seconds, peak, printed = fastest(command, directory: suite, same_output: false,
                                              environment: ENVIRONMENTS[engine])
    abort "the Rex suite failed under #{command.first}:\n#{printed}" unless printed.match?(/ 0 failures/)
    [engine, [seconds, peak]]
  end
  Row.new("rex_suite", results.transform_values(&:first), results.transform_values(&:last))
end

def wanted?(name) = ONLY.nil? || ONLY.include?(name)

def seconds_text(seconds) = seconds < 1 ? format("%.1f ms", seconds * 1000) : format("%.2f s", seconds)

def megabytes(bytes) = format("%.0f MB", bytes / 1_048_576.0)

# A row's name and times, printed as it is timed so a run that stops later
# keeps them.
def timed_text(row) = "#{row.name}: #{row.seconds.map { |engine, seconds| "#{engine} #{seconds_text(seconds)}" }.join(", ")}"

rows = []
scripts = [File.join(__dir__, "startup", "empty.rb")] + Dir[File.join(__dir__, "workloads", "*.rb")].sort +
          Dir[File.join(__dir__, "memory", "*.rb")].sort
scripts.each do |script|
  next unless wanted?(File.basename(script, ".rb"))
  rows << script_row(script)
  warn "timed #{timed_text(rows.last)}"
end
ruby_bench_benchmarks.each do |name, benchmark, needs_gems|
  next if needs_gems || !wanted?(name)
  rows << ruby_bench_row(name, benchmark)
  warn "timed #{timed_text(rows.last)}"
end
rows << rex_suite_row if wanted?("rex_suite")

puts "Metorex #{`#{METOREX} -V`.split.last}, #{RUBY_DESCRIPTION}"
puts
ratio_headings = ["Metorex ÷ CRuby", "Metorex ÷ YJIT"] + (NO_JIT ? ["Metorex ÷ no JIT"] : [])
headings = ENGINES.keys + ratio_headings + ENGINES.keys.map { |engine| "#{engine.sub("CRuby --yjit", "YJIT")} peak" }
puts "| Benchmark | #{headings.join(" | ")} |"
puts "| :-- | #{(["--:"] * headings.size).join(" | ")} |"
rows.each do |row|
  times = ENGINES.keys.map { |engine| seconds_text(row.seconds[engine]) }
  ratios = ["CRuby", "CRuby --yjit"].map { |engine| format("%.1f×", row.seconds["Metorex"] / row.seconds[engine]) }
  ratios << format("%.2f", row.seconds["Metorex"] / row.seconds["Metorex, no JIT"]) if NO_JIT
  peaks = ENGINES.keys.map { |engine| megabytes(row.peaks[engine]) }
  puts "| #{row.name} | #{(times + ratios + peaks).join(" | ")} |"
end
needing_gems = ruby_bench_benchmarks.select(&:last).map(&:first)
puts
puts "Not run, since they need gems installed with `bundle install`: #{needing_gems.join(", ")}."
