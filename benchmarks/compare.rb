# Times each workload under MRI and under metorex, with the peak memory of
# each run, and prints a table. The first row is an empty script, which times
# starting up.
# Run by hand from the repository root with a release build:
#
#   cargo build --release
#   ruby benchmarks/compare.rb
#
# Another metorex binary can be named as the first argument.

require "open3"

metorex = ARGV.fetch(0, File.expand_path("../target/release/metorex", __dir__))
abort "no metorex binary at #{metorex}" unless File.executable?(metorex)

# The flag that makes /usr/bin/time report the peak resident set size, and
# the line and unit it reports it in, which differ between macOS and Linux.
PEAK_MEMORY = if RUBY_PLATFORM.include?("darwin")
  { flag: "-l", pattern: /(\d+)\s+maximum resident set size/, bytes_per_unit: 1 }
else
  { flag: "-v", pattern: /Maximum resident set size \(kbytes\): (\d+)/, bytes_per_unit: 1024 }
end

# One run of `program` on `script`, answering the seconds it took, its peak
# resident set size in bytes, and what it printed.
def timed(program, script)
  started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  printed, reported, status = Open3.capture3("/usr/bin/time", PEAK_MEMORY[:flag], program, script)
  finished = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  abort "#{program} #{script} failed:\n#{printed}#{reported}" unless status.success?
  peak = reported[PEAK_MEMORY[:pattern], 1] or abort "no peak memory in:\n#{reported}"
  [finished - started, peak.to_i * PEAK_MEMORY[:bytes_per_unit], printed]
end

# The fastest of three runs, which leaves out a run slowed by something else
# on the machine, and the largest peak memory of the three.
def fastest(program, script)
  runs = Array.new(3) { timed(program, script) }
  outputs = runs.map(&:last).uniq
  abort "#{program} #{script} printed different things" unless outputs.size == 1
  [runs.map(&:first).min, runs.map { |run| run[1] }.max, outputs.first]
end

def megabytes(bytes) = format("%.1f MB", bytes / 1_048_576.0)

puts "metorex #{`#{metorex} -V`.split.last}, #{RUBY_DESCRIPTION}"
puts format("%-20s %10s %10s %8s %10s %12s", "workload", "MRI", "metorex", "ratio", "MRI peak",
            "metorex peak")
scripts = [File.join(__dir__, "startup", "empty.rb")] + Dir[File.join(__dir__, "workloads", "*.rb")].sort
scripts.each do |script|
  mri_seconds, mri_peak, mri_output = fastest(RbConfig.ruby, script)
  metorex_seconds, metorex_peak, metorex_output = fastest(metorex, script)
  abort "#{script} printed different things under MRI and metorex" unless mri_output == metorex_output
  puts format("%-20s %9.3fs %9.3fs %7.1fx %10s %12s", File.basename(script, ".rb"), mri_seconds,
              metorex_seconds, metorex_seconds / mri_seconds, megabytes(mri_peak), megabytes(metorex_peak))
end
