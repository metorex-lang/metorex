require "coverage"
Coverage.start(lines: true)
require_relative "conditional_lines_subject"
p Coverage.result.find { |path, _| path.end_with?("conditional_lines_subject.rb") }.last[:lines]
