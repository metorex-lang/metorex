# With Marshal patched for partial doubles, dumping an object that has a stub
# dumps a copy without it, and prints no warning.
require "rspec/mocks"
require "yaml"
$VERBOSE = true

Pair = Struct.new(:left, :right)
include RSpec::Mocks::ExampleMethods
RSpec::Mocks.setup
RSpec::Mocks.configuration.patch_marshal_to_support_partial_doubles = true
pair = Pair.new(7, "something")
before = Marshal.dump(pair)
allow(pair).to receive_messages(bazz: 5)
p Marshal.dump(pair) == before
RSpec::Mocks.configuration.patch_marshal_to_support_partial_doubles = false
RSpec::Mocks.teardown
