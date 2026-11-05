# Runs the spec files named on the command line, as the rspec command does,
# reading the options in the gem's .rspec.
require "rspec/core"

exit RSpec::Core::Runner.run(ARGV)
