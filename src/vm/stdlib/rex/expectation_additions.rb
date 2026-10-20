# What Rex adds to RSpec's own rspec-expectations, which it runs as RSpec
# writes it: the common matchers matched natively while `config.fast_paths`
# is on, a failure that names the expression handed to `expect` while
# `config.source_expressions` is on, and the diff settings `--no-diff` and
# `config.diff_line_limit`.

module Rex
  ExpectationNotMet = Expectations::ExpectationNotMetError

  module Expectations
    class ExpectationTarget
      # Where `expect` was called, kept when `config.source_expressions` is
      # on so a failure can name the expression it was handed.
      attr_accessor :source_location

      # `expect` builds its target here, so the call it was written in is
      # two frames up.
      module SourceLocation
        def for(value, block)
          target = super
          target.source_location = caller_locations(2, 1).first if ::Rex.configuration.source_expressions
          target
        end
      end
      singleton_class.prepend(SourceLocation)

      private

      def with_source_location
        held = Thread.current[:rex_expectation_location]
        Thread.current[:rex_expectation_location] = source_location
        yield
      ensure
        Thread.current[:rex_expectation_location] = held
      end
    end

    class ValueExpectationTarget
      def to(matcher = nil, message = nil, &block)
        # `eq`, `equal`, `be_truthy`, `be_nil`, `match`, and `include`
        # matched natively. It answers nil when the steps below have to run,
        # and false for a failure they then report.
        matched = __expect_to__(target, matcher, message, block, ::Rex::Matchers, self.class, PositiveExpectationHandler)
        return matched if matched
        if matched == false
          return with_source_location { ExpectationHelper.handle_failure(matcher, message, :failure_message) }
        end

        enforce_value_expectation(matcher)
        return super(matcher, message, &block) unless source_location

        with_source_location { super(matcher, message, &block) }
      end

      def not_to(matcher = nil, message = nil, &block)
        enforce_value_expectation(matcher)
        return super(matcher, message, &block) unless source_location

        with_source_location { super(matcher, message, &block) }
      end
      alias to_not not_to
    end

    class BlockExpectationTarget
      def to(matcher, message = nil, &block)
        enforce_block_expectation(matcher)
        return super(matcher, message, &block) unless source_location

        with_source_location { super(matcher, message, &block) }
      end

      def not_to(matcher, message = nil, &block)
        enforce_block_expectation(matcher)
        return super(matcher, message, &block) unless source_location

        with_source_location { super(matcher, message, &block) }
      end
      alias to_not not_to
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
        actual = Rex::Support::ObjectFormatter.format(matcher.actual)
        return nil if text.nil? || text == actual

        verb = failure_message_method == :failure_message ? "to" : "not to"
        "expected #{text} #{verb} #{matcher.description}, got #{actual}"
      end
    end

    module ExpectationHelper
      # RSpec's own, with the expression named in place of the matcher's
      # message when `config.source_expressions` kept one. Written over
      # rather than prepended, so a failure's backtrace has RSpec's frames.
      def self.handle_failure(matcher, message, failure_message_method)
        message = message.call if message.respond_to?(:call)
        message ||= SourceExpression.failure_message(matcher, failure_message_method)
        message ||= matcher.__send__(failure_message_method)

        if matcher.respond_to?(:diffable?) && matcher.diffable?
          ::Rex::Expectations.fail_with message, matcher.expected, matcher.actual
        else
          ::Rex::Expectations.fail_with message
        end
      end
    end

    # `--no-diff` leaves the diff out of an expectation's failure: `fail_with`
    # is handed a differ that finds nothing to show.
    class << self
      alias_method :__rspec_differ__, :differ

      def differ
        ::Rex.configuration.diff_enabled? ? __rspec_differ__ : NO_DIFF
      end
    end

    NO_DIFF = Object.new
    def NO_DIFF.diff(_actual, _expected)
      ""
    end
  end

  module Support
    class Differ
      # `config.diff_line_limit` leaves out the diff of two texts when either
      # runs longer than that many lines.
      module LineLimit
        def diff_as_string(actual, expected)
          limit = ::Rex.configuration.diff_line_limit
          return "" if limit && [actual.split("\n").size, expected.split("\n").size].max > limit

          super
        end
      end
      prepend LineLimit
    end
  end
end
