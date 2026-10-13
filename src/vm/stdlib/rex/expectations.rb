module Rex
  # Raised when an expectation fails. It descends from Exception rather than
  # StandardError, as RSpec's does, so a bare `rescue` in the code under test
  # does not swallow it.
  class ExpectationNotMet < Exception
  end

  module Expectations
    ExpectationNotMetError = Rex::ExpectationNotMet

    class << self
      def fail_with(message, expected = nil, actual = nil)
        unless message
          raise ArgumentError, "Failure message is nil. Does your matcher define the appropriate failure_message[_when_negated] method to return a string?"
        end

        message = Rex::Matchers::Differ.append_diff(message, expected, actual) if expected || actual
        if (aggregator = Thread.current[:rex_failure_aggregator])
          aggregator.call(ExpectationNotMet.new(message))
        else
          raise ExpectationNotMet, message
        end
      end

      def configuration
        @configuration ||= Core::ExpectationConfiguration.new(Rex.configuration)
      end
    end

    # What `expect(value)` returns.
    class ExpectationTarget
      UNDEFINED = Object.new.freeze

      attr_reader :target
      # Where `expect` was called, kept when `config.source_expressions` is
      # on so a failure can name the expression it was handed.
      attr_accessor :source_location

      def self.for(value, block)
        if UNDEFINED.equal?(value)
          raise ArgumentError, "You must pass either an argument or a block to `expect`." unless block

          BlockExpectationTarget.new(block)
        elsif block
          raise ArgumentError, "You cannot pass both an argument and a block to `expect`."
        else
          new(value)
        end
      end

      def initialize(value)
        @target = value
      end

      def to(matcher = nil, message = nil, &block)
        prevent_operator_matchers(:to) unless matcher
        enforce_value_expectation(matcher)
        with_source_location { PositiveExpectationHandler.handle_matcher(@target, matcher, message, &block) }
      end

      def not_to(matcher = nil, message = nil, &block)
        prevent_operator_matchers(:not_to) unless matcher
        enforce_value_expectation(matcher)
        with_source_location { NegativeExpectationHandler.handle_matcher(@target, matcher, message, &block) }
      end
      alias to_not not_to

      private

      # A matcher written for a block, such as `change`, handed a value.
      def enforce_value_expectation(matcher)
        return if matcher.nil? || !matcher.respond_to?(:supports_value_expectations?) || matcher.supports_value_expectations?

        formatted = Rex::Matchers::ObjectFormatter.format(matcher)
        Rex.deprecate(
          "expect(value).to #{formatted}",
          message: "The implicit block expectation syntax is deprecated, you should pass " \
                   "a block rather than an argument to `expect` to use the provided " \
                   "block expectation matcher or the matcher must implement " \
                   "`supports_value_expectations?`. e.g  `expect { value }.to " \
                   "#{formatted}` not `expect(value).to #{formatted}`"
        )
      end

      def with_source_location
        held = Thread.current[:rex_expectation_location]
        Thread.current[:rex_expectation_location] = source_location
        yield
      ensure
        Thread.current[:rex_expectation_location] = held
      end

      def prevent_operator_matchers(verb)
        raise ArgumentError, "The expect syntax does not support operator matchers, so you must pass a matcher to `##{verb}`."
      end
    end

    # What `expect { }` returns.
    class BlockExpectationTarget < ExpectationTarget
      def to(matcher = nil, message = nil, &block)
        enforce_block_expectation(matcher)
        super
      end

      def not_to(matcher = nil, message = nil, &block)
        enforce_block_expectation(matcher)
        super
      end
      alias to_not not_to

      private

      def enforce_value_expectation(_matcher); end

      def enforce_block_expectation(matcher)
        return if matcher.nil?
        return if matcher.respond_to?(:supports_block_expectations?) && matcher.supports_block_expectations?

        description = matcher.respond_to?(:description) ? matcher.description : matcher.inspect
        raise ExpectationNotMet, "You must pass an argument rather than a block to `expect` to use the provided " \
                                 "matcher (#{description}), or the matcher must implement " \
                                 "`supports_block_expectations?`."
      end
    end

    module ExpectationHelper
      def self.check_message(message)
        return if message.nil? || message.respond_to?(:to_str) || message.respond_to?(:call)

        Rex.warning("ignoring the provided expectation message argument(#{message.inspect}) since it is not a string or a proc")
      end

      def self.handle_failure(matcher, message, failure_message_method)
        message = message.call if message.respond_to?(:call)
        message ||= SourceExpression.failure_message(matcher, failure_message_method)
        message ||= matcher.__send__(failure_message_method)
        if matcher.respond_to?(:diffable?) && matcher.diffable?
          Expectations.fail_with(message, matcher.expected, matcher.actual)
        else
          Expectations.fail_with(message)
        end
      end
    end

    # With `config.source_expressions` on, a failure names the expression
    # handed to `expect` as it is written: `expected user.age to eq 42, got
    # 41`. A value written as a literal reads the same either way, so its
    # message stays as the matcher writes it.
    module SourceExpression
      def self.failure_message(matcher, failure_message_method)
        location = Thread.current[:rex_expectation_location]
        return nil unless location
        return nil unless matcher.respond_to?(:description) && matcher.respond_to?(:actual)

        require "rex/source"
        text = Rex::Source.__expect_argument__(File.expand_path(location.path), location.lineno)
        actual = Rex::Matchers::ObjectFormatter.format(matcher.actual)
        return nil if text.nil? || text == actual

        verb = failure_message_method == :failure_message ? "to" : "not to"
        "expected #{text} #{verb} #{matcher.description}, got #{actual}"
      end
    end

    class PositiveExpectationHandler
      def self.handle_matcher(actual, matcher, message = nil, &block)
        ExpectationHelper.check_message(message)
        Rex::Matchers.last_expectation_verb = verb
        Rex::Matchers.last_matcher = matcher
        return Rex::Matchers::BuiltIn::PositiveOperatorMatcher.new(actual) if matcher.nil?

        match = matcher.matches?(actual, &block)
        return match if match

        ExpectationHelper.handle_failure(matcher, message, :failure_message)
      end

      def self.verb
        "is expected to"
      end
    end

    class NegativeExpectationHandler
      def self.handle_matcher(actual, matcher, message = nil, &block)
        ExpectationHelper.check_message(message)
        Rex::Matchers.last_expectation_verb = verb
        Rex::Matchers.last_matcher = matcher
        return Rex::Matchers::BuiltIn::NegativeOperatorMatcher.new(actual) if matcher.nil?

        match = if matcher.respond_to?(:does_not_match?)
                  matcher.does_not_match?(actual, &block)
                else
                  !matcher.matches?(actual, &block)
                end
        return match if match

        ExpectationHelper.handle_failure(matcher, message, :failure_message_when_negated)
      end

      def self.verb
        "is expected not to"
      end
    end

    # `should` and `should_not` on every object, on when a suite asks for
    # the old syntax.
    module Syntax
      module ShouldMethods
        def should(matcher = nil, message = nil, &block)
          Rex::Matchers.last_should_verb = true
          PositiveExpectationHandler.handle_matcher(self, matcher, message, &block)
        end

        def should_not(matcher = nil, message = nil, &block)
          Rex::Matchers.last_should_verb = true
          NegativeExpectationHandler.handle_matcher(self, matcher, message, &block)
        end
      end

      def self.enable_should
        return if @should_enabled

        @should_enabled = true
        BasicObject.__send__(:include, ShouldMethods)
      end

      def self.disable_should
        return unless @should_enabled

        @should_enabled = false
        ShouldMethods.instance_methods(false).each do |name|
          ShouldMethods.__send__(:remove_method, name)
        end
      end
    end
  end
end
