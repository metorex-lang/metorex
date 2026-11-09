Dir.chdir(__dir__)
ARGV.replace(["--format", "progress", "spec/helper_spec.rb"])
require "rspec/core"
RSpec::Core::Runner.invoke
