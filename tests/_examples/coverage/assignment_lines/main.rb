require "coverage"
Coverage.start(lines: true)
load File.join(__dir__, "empty.rb")
load File.join(__dir__, "settings.rb")
result = Coverage.result
p result[File.join(__dir__, "empty.rb")][:lines]
p result[File.join(__dir__, "settings.rb")][:lines]
