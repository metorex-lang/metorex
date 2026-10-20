# `require "rspec/mocks/standalone"` extends `main` with the mocks API, even
# when the require is written in a block run against another object.
main = self
holder = Object.new
holder.instance_exec { require "rspec/mocks/standalone" }
p main.singleton_class.include?(RSpec::Mocks::ExampleMethods)
p holder.respond_to?(:double)

main.instance_eval do
  crate = double
  expect(crate).to receive(:weigh).and_return(12)
  p crate.weigh
  RSpec::Mocks.verify
  RSpec::Mocks.teardown
end
