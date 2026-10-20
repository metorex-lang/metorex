require 'rspec/mocks'

module Rex
  module Mocks
    # @private
    module MinitestIntegration
      include ::Rex::Mocks::ExampleMethods

      def before_setup
        ::Rex::Mocks.setup
        super
      end

      def after_teardown
        super

        # Only verify if there's not already an error. Otherwise
        # we risk getting the same failure twice, since negative
        # expectation violations raise both when the message is
        # unexpectedly received, and also during `verify` (in case
        # the first failure was caught by user code via a
        # `rescue Exception`).
        ::Rex::Mocks.verify unless failures.any?
      ensure
        ::Rex::Mocks.teardown
      end
    end
  end
end

Minitest::Test.send(:include, Rex::Mocks::MinitestIntegration)

if defined?(::Minitest::Expectation)
  if defined?(::Rex::Expectations) && ::Minitest::Expectation.method_defined?(:to)
    # rspec/expectations/minitest_integration has already been loaded and
    # has defined `to`/`not_to`/`to_not` on `Minitest::Expectation` so we do
    # not want to here (or else we would interfere with rspec-expectations' definition).
  else
    # ...otherwise, define those methods now. If `rspec/expectations/minitest_integration`
    # is loaded after this file, it'll override the definition here.
    Minitest::Expectation.class_eval do
      include Rex::Mocks::ExpectationTargetMethods

      def to(*args)
        ctx.assertions += 1
        super
      end

      def not_to(*args)
        ctx.assertions += 1
        super
      end

      def to_not(*args)
        ctx.assertions += 1
        super
      end
    end
  end
end

module Rex
  module Mocks
    remove_const :MockExpectationError
    # Raised when a message expectation is not satisfied.
    MockExpectationError = ::Minitest::Assertion
  end
end
