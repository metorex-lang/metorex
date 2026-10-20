# A message chain expected on any instance of Object is met by calling the
# whole chain on one object.
require "rspec/mocks"

include RSpec::Mocks::ExampleMethods
RSpec::Mocks.setup
expect_any_instance_of(Object).to receive_message_chain(:to_a, :length => 3)
p Object.new.to_a.length
RSpec::Mocks.verify
RSpec::Mocks.teardown
puts "verified"
