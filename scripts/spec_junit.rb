# Writes a JUnit report of one pass of scripts/run_ruby_spec.sh, a test case
# for each mspec run in its queue, so Jenkins lists the spec files that failed
# with what mspec printed for them.
#
#   ruby scripts/spec_junit.rb REPORT SUITE WORK_DIR SPEC_DIR
#
# WORK_DIR holds the runner's `manifest`, a line for each run of the pass with
# its index and its paths joined by tabs, and for each index the run's output
# in `<index>.out` and its exit status in `<index>.code`.

report, suite, work_dir, spec_dir = ARGV
abort "usage: ruby scripts/spec_junit.rb REPORT SUITE WORK_DIR SPEC_DIR" unless spec_dir

SUMMARY = /^(\d+) files?, (\d+) examples?, (\d+) expectations?, (\d+) failures?, (\d+) errors?, (\d+) tagged$/
# What mspec prints for every run, which says nothing about a failure.
FRAMING = [/^\$ /, /^metorex( [0-9][0-9.]*)? \(ruby-compatible\)/, /^Finished in [0-9.]+ seconds$/, SUMMARY].freeze

# Text XML 1.0 can hold, with what it cannot replaced.
def xml_text(text)
  text.encode("UTF-8", invalid: :replace, undef: :replace)
      .gsub(/[^\t\n\r -퟿-�]/, "?")
      .gsub("&", "&amp;").gsub("<", "&lt;").gsub(">", "&gt;").gsub('"', "&quot;")
end

cases = File.readlines(File.join(work_dir, "manifest"), chomp: true).map do |line|
  index, *paths = line.split("\t")
  output = File.exist?(File.join(work_dir, "#{index}.out")) ? File.binread(File.join(work_dir, "#{index}.out")) : ""
  code = File.exist?(File.join(work_dir, "#{index}.code")) ? File.read(File.join(work_dir, "#{index}.code")).strip : "missing"
  relative = paths.map { |path| path.delete_prefix("#{spec_dir}/") }
  summary = output.scrub.lines.grep(SUMMARY).last
  failure = if code != "0"
    summary ? summary.strip : "mspec exited #{code}"
  end
  {
    classname: File.dirname(relative.first).tr("/", "."),
    name: relative.map { |path| File.basename(path) }.join(" + "),
    failure: failure,
    output: output.scrub.lines.reject { |text| text.strip.empty? || FRAMING.any? { |pattern| text.match?(pattern) } }.join
  }
end

failures = cases.count { |test| test[:failure] }
File.open(report, "w") do |file|
  file.puts %(<?xml version="1.0" encoding="UTF-8"?>)
  file.puts %(<testsuites>)
  file.puts %(  <testsuite name="#{xml_text(suite)}" tests="#{cases.size}" failures="#{failures}" errors="0">)
  cases.each do |test|
    attributes = %(classname="#{xml_text(test[:classname])}" name="#{xml_text(test[:name])}")
    if test[:failure]
      file.puts %(    <testcase #{attributes}>)
      file.puts %(      <failure message="#{xml_text(test[:failure])}">#{xml_text(test[:output])}</failure>)
      file.puts %(    </testcase>)
    else
      file.puts %(    <testcase #{attributes}/>)
    end
  end
  file.puts %(  </testsuite>)
  file.puts %(</testsuites>)
end
