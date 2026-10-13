require "coverage"
Coverage.start(lines: true)
require_relative "conditional_lines_subject"
subject = Coverage.__coverage_branches__.find { |path, _| path.end_with?("conditional_lines_subject.rb") }
subject.last.each do |kind, line, arms|
  puts "#{kind} at #{line}: " + arms.map { |arm_kind, arm_line, count| "#{arm_kind}@#{arm_line}=#{count}" }.join(" ")
end
