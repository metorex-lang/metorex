# Keeps the counts each file of a suite reaches under MRI and under Metorex,
# and compares the two: RSpec's own spec files in
# scripts/rspec_suite_results.tsv, or the file SUITE_RESULTS names, as
# scripts/run_gem_suites.sh names scripts/gem_suite_results.tsv.
#
#   ruby scripts/suite_results.rb record ENGINE RESULTS
#   ruby scripts/suite_results.rb differences [RESULTS]
#   ruby scripts/suite_results.rb regressions RESULTS
#   ruby scripts/suite_results.rb table
#
# ENGINE is mri or metorex. RESULTS is the file scripts/run_rspec_suite.sh or
# scripts/run_gem_suites.sh writes, a file's path and the summary line its run
# printed, a tab apart: RSpec's, test-unit's, or minitest's. A failure counts
# an error with it, and a pending test counts an omission or a skip. `record` stores those counts as the engine's. `differences` lists
# the files whose Metorex counts differ from MRI's, from RESULTS when it is
# given and from the stored counts otherwise, and exits 1 when there are any.
# `regressions` compares RESULTS with the stored Metorex counts instead: it
# lists each file that passes fewer examples than it did, and exits 1 when
# there are any, and prints the stored row to write for each file that passes
# more. `table` prints the README's table of each suite's totals.

STORE = ENV.fetch("SUITE_RESULTS", File.expand_path("rspec_suite_results.tsv", __dir__))
ENGINES = %w[mri metorex].freeze
COLUMNS = %w[examples failures pending].freeze
HEADER = ["file", *ENGINES.flat_map { |engine| COLUMNS.map { |column| "#{engine}_#{column}" } }].freeze

# The counts a summary line names, or nil for a file that did not finish.
def counts(summary)
  case summary
  when /\A(\d+) examples?, (\d+) failures?/
    [$1.to_i, $2.to_i, summary[/(\d+) pending/, 1].to_i]
  when /\A(\d+) tests, \d+ assertions, (\d+) failures, (\d+) errors, (\d+) pendings, (\d+) omissions/
    [$1.to_i, $2.to_i + $3.to_i, $4.to_i + $5.to_i]
  when /\A(\d+) runs, \d+ assertions, (\d+) failures, (\d+) errors, (\d+) skips/
    [$1.to_i, $2.to_i + $3.to_i, $4.to_i]
  end
end

def read_results(path)
  File.readlines(path, chomp: true).to_h do |line|
    file, summary = line.split("\t", 2)
    [file, counts(summary.to_s)]
  end
end

def read_store
  return {} unless File.exist?(STORE)
  rows = File.readlines(STORE, chomp: true).drop(1).map { |line| line.split("\t") }
  rows.to_h do |file, *fields|
    by_engine = ENGINES.each_with_index.to_h do |engine, index|
      values = fields[index * 3, 3]
      [engine, values.include?("-") ? nil : values.map(&:to_i)]
    end
    [file, by_engine]
  end
end

def write_store(store)
  lines = [HEADER.join("\t")]
  store.sort.each do |file, by_engine|
    fields = ENGINES.flat_map { |engine| by_engine[engine] || %w[- - -] }
    lines << [file, *fields].join("\t")
  end
  File.write(STORE, lines.join("\n") + "\n")
end

def passing(found)
  found ? found[0] - found[1] - found[2] : -1
end

def describe(found)
  found ? format("%d examples, %d failures, %d pending", *found) : "did not finish"
end

command, *arguments = ARGV
case command
when "record"
  engine, results = arguments
  abort "usage: ruby scripts/suite_results.rb record mri|metorex RESULTS" unless ENGINES.include?(engine) && results
  store = read_store
  read_results(results).each do |file, found|
    store[file] ||= {}
    store[file][engine] = found
  end
  write_store(store)
when "differences"
  store = read_store
  metorex = arguments.first ? read_results(arguments.first) : store.transform_values { |by_engine| by_engine["metorex"] }
  differing = metorex.sort.reject { |file, found| store.dig(file, "mri") == found }
  differing.each do |file, found|
    puts "#{file}: MRI #{describe(store.dig(file, "mri"))}, Metorex #{describe(found)}"
  end
  exit(differing.empty? ? 0 : 1)
when "regressions"
  abort "usage: ruby scripts/suite_results.rb regressions RESULTS" unless arguments.first
  store = read_store
  fell = []
  read_results(arguments.first).sort.each do |file, found|
    recorded = store.dig(file, "metorex")
    next if passing(found) == passing(recorded)
    if passing(found) < passing(recorded)
      fell << file
      puts "#{file}: recorded #{describe(recorded)}, now #{describe(found)}"
    else
      fields = ENGINES.flat_map { |engine| (engine == "metorex" ? found : store.dig(file, engine)) || %w[- - -] }
      puts "#{file} passes more than recorded, update its row to:"
      puts [file, *fields].join("\t")
    end
  end
  exit(fell.empty? ? 0 : 1)
when "table"
  totals = Hash.new { |hash, suite| hash[suite] = Hash.new(0) }
  read_store.each do |file, by_engine|
    suite = file.split("/").first
    ENGINES.each do |engine|
      examples, failures, pending = by_engine[engine] || [0, 0, 0]
      totals[suite]["#{engine}_passing"] += examples - failures - pending
      totals[suite]["#{engine}_examples"] += examples
      totals[suite]["#{engine}_pending"] += pending
    end
  end
  puts "| Suite | Passing under MRI | Passing under Metorex | Examples | Pending |"
  puts "| :-- | --: | --: | --: | --: |"
  totals.sort.each do |suite, total|
    puts "| #{suite} | #{total["mri_passing"]} | #{total["metorex_passing"]} | #{total["mri_examples"]} | #{total["mri_pending"]} |"
  end
else
  abort "usage: ruby scripts/suite_results.rb record|differences|regressions|table"
end
