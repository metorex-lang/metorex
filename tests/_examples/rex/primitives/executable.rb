require "rspec/core"

path = RSpec::Core.path_to_executable
puts File.executable?(path)
puts File.read(path).include?("RSpec::Core::Runner.invoke")
puts RSpec::Core.path_to_executable.equal?(path)
puts RSpec.const_missing(:Mocks).equal?(RSpec::Mocks)
