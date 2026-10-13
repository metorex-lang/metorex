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

        message = Rex::Matchers::Differ.append_diff(message, expected, actual)
        Rex::Support.notify_failure(ExpectationNotMet.new(message))
      end

      def configuration
        @configuration ||= Configuration.new
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

    # Where `should`, `should_not`, and `expect` are defined, and whether
    # each is.
    module Syntax
      module_function

      def default_should_host
        @default_should_host ||= ::Object.ancestors.last
      end

      def warn_about_should!
        @warn_about_should = true
      end

      def warn_about_should_unless_configured(method_name)
        return unless @warn_about_should

        Rex.deprecate(
          "Using `#{method_name}` from rspec-expectations' old `:should` syntax without explicitly enabling the syntax",
          replacement: "the new `:expect` syntax or explicitly enable `:should` with `config.expect_with(:rspec) { |c| c.syntax = :should }`"
        )
        @warn_about_should = false
      end

      def enable_should(syntax_host = default_should_host)
        @warn_about_should = false if syntax_host == default_should_host
        return if should_enabled?(syntax_host)

        syntax_host.module_exec do
          def should(matcher = nil, message = nil, &block)
            ::Rex::Expectations::Syntax.warn_about_should_unless_configured(::Kernel.__method__)
            ::Rex::Matchers.last_should_verb = true
            ::Rex::Expectations::PositiveExpectationHandler.handle_matcher(self, matcher, message, &block)
          end

          def should_not(matcher = nil, message = nil, &block)
            ::Rex::Expectations::Syntax.warn_about_should_unless_configured(::Kernel.__method__)
            ::Rex::Matchers.last_should_verb = true
            ::Rex::Expectations::NegativeExpectationHandler.handle_matcher(self, matcher, message, &block)
          end
        end
      end

      def disable_should(syntax_host = default_should_host)
        return unless should_enabled?(syntax_host)

        syntax_host.module_exec do
          undef should
          undef should_not
        end
      end

      def enable_expect(syntax_host = ::Rex::Matchers)
        return if expect_enabled?(syntax_host)

        syntax_host.module_exec do
          def expect(value = ::Rex::Expectations::ExpectationTarget::UNDEFINED, &block)
            target = ::Rex::Expectations::ExpectationTarget.for(value, block)
            target.source_location = caller_locations(1, 1).first if ::Rex.configuration.source_expressions
            target
          end
        end
      end

      def disable_expect(syntax_host = ::Rex::Matchers)
        return unless expect_enabled?(syntax_host)

        syntax_host.module_exec do
          undef expect
        end
      end

      def should_enabled?(syntax_host = default_should_host)
        syntax_host.method_defined?(:should)
      end

      def expect_enabled?(syntax_host = ::Rex::Matchers)
        syntax_host.method_defined?(:expect)
      end
    end

    # What `config.expect_with(:rspec) { |c| }` yields, and
    # `RSpec::Expectations.configuration`.
    class Configuration
      FALSE_POSITIVE_BEHAVIOURS = {
        warn: ->(message) { Rex.warning(message) },
        raise: ->(message) { raise ArgumentError, message },
        nothing: ->(_message) { true }
      }.freeze

      def initialize
        @on_potential_false_positives = :warn
        @strict_predicate_matchers = false
      end

      def syntax=(values)
        if Array(values).include?(:expect)
          Expectations::Syntax.enable_expect
        else
          Expectations::Syntax.disable_expect
        end

        if Array(values).include?(:should)
          Expectations::Syntax.enable_should
        else
          Expectations::Syntax.disable_should
        end
      end

      def syntax
        syntaxes = []
        syntaxes << :should if Expectations::Syntax.should_enabled?
        syntaxes << :expect if Expectations::Syntax.expect_enabled?
        syntaxes
      end

      # Rex reads the length where it formats, so the setting is kept here
      # rather than on a formatter instance.
      attr_reader :max_formatted_output_length

      def max_formatted_output_length=(length)
        @max_formatted_output_length = length
        @max_formatted_output_length_set = true
      end

      def max_formatted_output_length_set?
        @max_formatted_output_length_set ? true : false
      end

      def color?
        ::Rex.configuration.color_enabled?
      end

      def add_should_and_should_not_to(*modules)
        modules.each do |mod|
          Expectations::Syntax.enable_should(mod)
        end
      end

      attr_writer :backtrace_formatter

      def backtrace_formatter
        @backtrace_formatter ||= ::Rex.configuration.backtrace_formatter
      end

      attr_writer :include_chain_clauses_in_custom_matcher_descriptions

      def include_chain_clauses_in_custom_matcher_descriptions?
        @include_chain_clauses_in_custom_matcher_descriptions ||= false
      end
      alias include_chain_clauses_in_custom_matcher_descriptions include_chain_clauses_in_custom_matcher_descriptions?

      def reset_syntaxes_to_default
        self.syntax = [:should, :expect]
        Rex::Expectations::Syntax.warn_about_should!
      end

      NullBacktraceFormatter = Module.new do
        def self.format_backtrace(backtrace)
          backtrace
        end
      end

      def warn_about_potential_false_positives=(boolean)
        if boolean
          self.on_potential_false_positives = :warn
        elsif warn_about_potential_false_positives?
          self.on_potential_false_positives = :nothing
        end
      end

      attr_reader :on_potential_false_positives

      def on_potential_false_positives=(behavior)
        unless FALSE_POSITIVE_BEHAVIOURS.key?(behavior)
          raise ArgumentError, "Supported values are: #{FALSE_POSITIVE_BEHAVIOURS.keys}"
        end

        @on_potential_false_positives = behavior
      end

      attr_reader :strict_predicate_matchers

      def strict_predicate_matchers=(value)
        raise ArgumentError, "Pass `true` or `false`" unless value == true || value == false

        @strict_predicate_matchers = value
      end

      def strict_predicate_matchers?
        @strict_predicate_matchers
      end

      def warn_about_potential_false_positives?
        on_potential_false_positives == :warn
      end

      def false_positives_handler
        FALSE_POSITIVE_BEHAVIOURS.fetch(@on_potential_false_positives)
      end
    end
  end
end
