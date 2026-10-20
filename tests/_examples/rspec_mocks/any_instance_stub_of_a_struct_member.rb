# A Struct member's reader can be stubbed on any instance, and reads the
# member again once the stub is taken down.
require "rspec/mocks"

Pallet = Struct.new(:label)
include RSpec::Mocks::ExampleMethods
RSpec::Mocks.setup
pallet = Pallet.new(:bar)
allow_any_instance_of(Pallet).to receive(:label).and_return(:baz)
p pallet.label
RSpec::Mocks.space.verify_all
RSpec::Mocks.teardown
p pallet.label
