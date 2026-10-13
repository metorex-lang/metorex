module Rex
  module Mocks
    # Raised when a double is used in a way its expectations forbid. It is an
    # Exception rather than a StandardError, as RSpec's is, so code under test
    # rescuing StandardError does not swallow it.
    class MockExpectationError < Exception
    end

    class ExpiredTestDoubleError < MockExpectationError
    end

    class OutsideOfExampleError < StandardError
    end

    class NegationUnsupportedError < StandardError
    end

    class UnsupportedMatcherError < StandardError
    end

    class VerifyingDoubleNotDefinedError < StandardError
    end

    class << self
      # The doubles and stubs of the example running now, which the example
      # verifies and then takes down when it ends.
      def space
        Thread.current[:rex_mock_space] ||= Space.new
      end

      # A fast path: an example that made no double, stub, or constant
      # change verifies and takes down nothing, and its space is made only
      # when it first needs one. `__mock_space_idle__` says when that holds.
      def setup
        Thread.current[:rex_mock_space] = __mock_space_idle__(nil, Space) ? nil : Space.new
      end

      def verify
        return if __mock_space_idle__(Thread.current[:rex_mock_space], Space)

        space.verify_all
      end

      def teardown
        space.reset_all unless __mock_space_idle__(Thread.current[:rex_mock_space], Space)
      ensure
        Thread.current[:rex_mock_space] = nil
      end


      def allow_message(subject, message, opts = {}, &block)
        space.proxy_for(subject).add_stub(message, opts, &block)
      end

      def expect_message(subject, message, opts = {}, &block)
        space.proxy_for(subject).add_message_expectation(message, opts, &block)
      end
    end

    # The caller line a double was set up from, which a failure found when
    # the example ends points at.
    def self.caller_line
      frame = caller_locations.find { |location| !Rex::Core.framework_location?(location) }
      frame ? "#{frame.path}:#{frame.lineno}:in '#{frame.label}'" : nil
    end

    class Space
      attr_reader :proxies, :any_instance_recorders

      def initialize
        @proxies = {}
        @any_instance_recorders = {}
        @constant_mutators = []
      end

      def register_constant_mutator(mutator)
        @constant_mutators << mutator
      end

      def constant_mutator_for(name)
        @constant_mutators.find { |mutator| mutator.full_constant_name == name }
      end

      def registered?(object)
        @proxies.key?(object.__id__)
      end

      def proxy_for(object)
        id = object.__id__
        @proxies[id] ||= begin
          proxy = if TestDouble === object
                    object.__send__(:__build_proxy)
                  elsif Module === object && Mocks.configuration.verify_partial_doubles?
                    VerifyingPartialDoubleProxy.new(object)
                  elsif Mocks.configuration.verify_partial_doubles?
                    VerifyingPartialDoubleProxy.new(object)
                  else
                    PartialDoubleProxy.new(object)
                  end
          proxy
        end
      end

      def proxy_registered?(object)
        @proxies.key?(object.__id__)
      end

      def registered_proxy(object)
        @proxies[object.__id__]
      end

      def any_instance_recorder_for(klass, only_return_existing = false)
        return @any_instance_recorders[klass.__id__] if only_return_existing

        @any_instance_recorders[klass.__id__] ||= AnyInstance::Recorder.new(klass)
      end

      def any_instance_proxy_for(klass)
        AnyInstance::Proxy.new(any_instance_recorder_for(klass), proxies_of(klass))
      end

      def proxies_of(klass)
        @proxies.values.select { |proxy| klass === proxy.object }
      end

      def ensure_registered(object)
        proxy_for(object)
      end

      def any_instance_recorders_from_ancestry_of(object)
        Rex::Support.class_of(object).ancestors.filter_map do |klass|
          @any_instance_recorders[klass.__id__]
        end
      end

      def verify_all
        @proxies.each_value(&:verify)
        @any_instance_recorders.each_value(&:verify)
      end

      def reset_all
        @proxies.each_value(&:reset)
        @any_instance_recorders.each_value(&:stop_all_observation!)
        @proxies.clear
        @any_instance_recorders.clear
      ensure
        @constant_mutators.reverse_each(&:idempotently_reset)
        @constant_mutators.clear
      end
    end

    # Writes the failure messages RSpec writes, word for word.
    class ErrorGenerator
      attr_writer :opts

      def initialize(target = nil)
        @target = target
      end

      def opts
        @opts ||= {}
      end

      def raise_unexpected_message_error(message, args)
        __raise "#{intro} received unexpected message :#{message} with #{format_args(args)}"
      end

      def raise_unexpected_message_args_error(expectation, args_for_multiple_calls, backtrace_line = nil)
        __raise error_message(expectation, args_for_multiple_calls), backtrace_line
      end

      def raise_missing_default_stub_error(expectation, args_for_multiple_calls)
        __raise(error_message(expectation, args_for_multiple_calls) +
          "\n Please stub a default value first if message might be received with other args as well. ")
      end

      def raise_similar_message_args_error(expectation, args_for_multiple_calls, backtrace_line = nil)
        __raise error_message(expectation, args_for_multiple_calls), backtrace_line
      end

      def raise_expectation_error(message, expected_received_count, argument_list_matcher,
                                  actual_received_count, expectation_count_type, args,
                                  backtrace_line = nil)
        expected_part = expected_part_of_expectation_error(expected_received_count, expectation_count_type, argument_list_matcher)
        received_part = received_part_of_expectation_error(actual_received_count, args)
        __raise "(#{intro(:unwrapped)}).#{message}#{format_args(args)}\n    #{expected_part}\n    #{received_part}", backtrace_line
      end

      def raise_expectation_on_unstubbed_method(method)
        __raise "#{intro} expected to have received #{method}, but that " \
                "object is not a spy or method has not been stubbed."
      end

      def raise_expectation_on_mocked_method(method)
        __raise "#{intro} expected to have received #{method}, but that " \
                "method has been mocked instead of stubbed or spied."
      end

      def raise_double_negation_error(wrapped_expression)
        __raise "Isn't life confusing enough? You've already set a " \
                "negative message expectation and now you are trying to " \
                "negate it again with `never`. What does an expression like " \
                "`#{wrapped_expression}.not_to receive(:msg).never` even mean?"
      end

      def raise_wrong_arity_error(args_to_yield, signature)
        __raise "#{intro} yielded |#{arg_list(args_to_yield)}| to block with #{signature.description}"
      end

      def raise_missing_block_error(args_to_yield)
        __raise "#{intro} asked to yield |#{arg_list(args_to_yield)}| but no block was passed"
      end

      def raise_non_public_error(method_name, visibility)
        raise NoMethodError, format("%s method `%s' called on %s", visibility, method_name, intro)
      end

      def raise_out_of_order_error(message)
        __raise "#{intro} received :#{message} out of order"
      end

      def raise_unimplemented_error(doubled_module, method_name, object)
        message = case object
                  when InstanceVerifyingDouble
                    "the %s class does not implement the instance method: %s" + (doubled_module.respond_to?(:name) && doubled_module.name ? "" : "")
                  when ClassVerifyingDouble
                    "the %s class does not implement the class method: %s"
                  else
                    "%s does not implement: %s"
                  end
        __raise(format(message, doubled_module.description, method_name))
      end

      def raise_verifying_double_not_defined_error(ref)
        notify(VerifyingDoubleNotDefinedError.new(
                 "#{ref.description.inspect} is not a defined constant. " \
                 "Perhaps you misspelt it? " \
                 "Disable check with `verify_doubled_constant_names` configuration option."
               ))
      end

      def default_error_message(expectation, expected_args, actual_args)
        "#{intro} received #{expectation.message.inspect} #{unexpected_arguments_message(expected_args, actual_args)}"
      end

      def describe_expectation(verb, message, expected_received_count, _actual_received_count, args)
        "#{verb} #{message}#{format_args(args)} #{count_message(expected_received_count)}"
      end

      def intro(unwrapped = false)
        case @target
        when TestDouble then TestDoubleFormatter.format(@target, unwrapped)
        when Class
          formatted = "#{@target.inspect} (class)"
          return formatted if unwrapped

          "#<#{formatted}>"
        when NilClass then "nil"
        else @target.inspect
        end
      end

      def format_args(args)
        return "(no args)" if args.empty?

        "(#{arg_list(args)})"
      end

      def arg_list(args)
        args.map { |arg| Rex::Matchers::ObjectFormatter.format(arg) }.join(", ")
      end

      def notify(*args)
        error = args.first
        raise error
      end

      private

      def error_message(expectation, args_for_multiple_calls)
        expected_args = format_args(expectation.expected_args)
        actual_args = args_for_multiple_calls.map { |args| format_args(args) }.join(", ")
        message = default_error_message(expectation, expected_args, actual_args)
        if args_for_multiple_calls.one?
          diff = Rex::Matchers::Differ.diff(args_for_multiple_calls.first, expectation.expected_args)
          message += "\nDiff:#{diff}" unless diff.strip.empty?
        end
        message
      end

      def unexpected_arguments_message(expected_args_string, actual_args_string)
        "with unexpected arguments\n  expected: #{expected_args_string}\n       got: #{actual_args_string}"
      end

      def expected_part_of_expectation_error(expected_received_count, expectation_count_type, argument_list_matcher)
        "expected: #{count_message(expected_received_count, expectation_count_type)}" +
          method_call_args_description(argument_list_matcher.expected_args) { argument_list_matcher.expected_args.length > 0 }.to_s
      end

      def received_part_of_expectation_error(actual_received_count, args)
        "received: #{count_message(actual_received_count)}" +
          method_call_args_description(args) { actual_received_count > 0 && args.length > 0 }.to_s
      end

      def method_call_args_description(args, generic_prefix = " with arguments: ", matcher_prefix = " with ")
        case args.first
        when ArgumentMatchers::AnyArgsMatcher then "#{matcher_prefix}any arguments"
        when ArgumentMatchers::NoArgsMatcher then "#{matcher_prefix}no arguments"
        else
          "#{generic_prefix}#{format_args(args)}" if yield
        end
      end

      def count_message(count, expectation_count_type = nil)
        return "at least #{times(count.abs)}" if count < 0 || expectation_count_type == :at_least
        return "at most #{times(count)}" if expectation_count_type == :at_most

        times(count)
      end

      def times(count)
        "#{count} time#{count.to_s == '1' ? '' : 's'}"
      end

      def __raise(message, backtrace_line = nil)
        message = opts[:message] unless opts[:message].nil?
        exception = MockExpectationError.new(message)
        if backtrace_line
          exception.set_backtrace([backtrace_line] + caller)
        end
        Rex::Support.notify_failure(exception)
      end
    end

    module TestDoubleFormatter
      def self.format(double, unwrapped = false)
        text = "#{type_desc(double)}#{verified_module_desc(double)} #{name_desc(double)}"
        return text if unwrapped

        "#<#{text}>"
      end

      def self.type_desc(double)
        case double
        when InstanceVerifyingDouble then "InstanceDouble"
        when ClassVerifyingDouble then "ClassDouble"
        when ObjectVerifyingDouble then "ObjectDouble"
        else "Double"
        end
      end

      # Only a verifying double names the class it stands for. Asking a
      # double whether it answers would reach a null object's method_missing.
      def self.verified_module_desc(double)
        return nil unless VerifyingDouble === double

        "(#{double.__send__(:__doubled_reference).description})"
      end

      def self.name_desc(double)
        name = double.__send__(:__double_name)
        name ? name.inspect : "(anonymous)"
      end
    end

    # A double: an object that answers the messages it was told to and
    # raises for any other.
    class TestDouble
      def initialize(name = nil, stubs = {})
        @__expired = false
        if Hash === name && stubs.empty?
          stubs = name
          @name = nil
        else
          @name = name
        end
        assign_stubs(stubs)
      end

      def as_null_object
        __mock_proxy.as_null_object
      end

      def null_object?
        __mock_proxy.null_object?
      end

      def freeze
        self
      end

      def ==(other)
        other == __mock_proxy
      end

      def inspect
        TestDoubleFormatter.format(self)
      end

      def to_s
        inspect.tr("<", "[").tr(">", "]")
      end

      def respond_to?(message, include_private = false)
        __mock_proxy.null_object? ? true : super
      end

      def __build_proxy
        TestDoubleProxy.new(self, ErrorGenerator.new(self))
      end

      def __disallow_further_usage!
        @__expired = true
      end

      private

      def __double_name
        @name
      end

      def method_missing(message, *args, &block)
        proxy = __mock_proxy
        proxy.record_message_received(message, *args, &block)

        if proxy.null_object?
          case message
          when :to_int then return 0
          when :to_a, :to_ary then return nil
          when :to_str then return to_s
          else return self
          end
        end

        # Defined private and protected methods behave as Ruby's own do.
        if private_methods.include?(message) || protected_methods.include?(message)
          return super
        end

        proxy.raise_unexpected_message_error(message, args)
      end
      ruby2_keywords(:method_missing)

      def respond_to_missing?(*)
        __mock_proxy.null_object?
      end

      def assign_stubs(stubs)
        stubs.each_pair do |message, response|
          __mock_proxy.add_simple_stub(message, response)
        end
      end

      def __mock_proxy
        Mocks.space.proxy_for(self)
      end
    end

    class Double < TestDouble
    end

    # The rules for matching the arguments a message was sent with.
    module ArgumentMatchers
      def any_args
        AnyArgsMatcher::INSTANCE
      end

      def anything
        AnyArgMatcher::INSTANCE
      end

      def no_args
        NoArgsMatcher::INSTANCE
      end

      def duck_type(*args)
        DuckTypeMatcher.new(*args)
      end

      def boolean
        BooleanMatcher::INSTANCE
      end

      def hash_including(*args)
        HashIncludingMatcher.new(ArgumentMatchers.anythingize_lonely_keys(*args))
      end

      def hash_excluding(*args)
        HashExcludingMatcher.new(ArgumentMatchers.anythingize_lonely_keys(*args))
      end
      alias hash_not_including hash_excluding

      def array_including(*args)
        actually_an_array = Array === args.first && args.count == 1 ? args.first : args
        ArrayIncludingMatcher.new(actually_an_array)
      end

      def array_excluding(*args)
        actually_an_array = Array === args.first && args.count == 1 ? args.first : args
        ArrayExcludingMatcher.new(actually_an_array)
      end

      def instance_of(klass)
        InstanceOf.new(klass)
      end
      alias an_instance_of instance_of

      def kind_of(klass)
        KindOf.new(klass)
      end
      alias a_kind_of kind_of

      def self.anythingize_lonely_keys(*args)
        hash = Hash === args.last ? args.delete_at(-1) : {}
        args.each { |arg| hash[arg] = AnyArgMatcher::INSTANCE }
        hash
      end

      class SingletonMatcher
        private_class_method :new

        def self.inherited(subklass)
          super
          subklass.const_set(:INSTANCE, subklass.send(:new))
        end
      end

      class AnyArgsMatcher < SingletonMatcher
        def description
          "*(any args)"
        end

        def inspect
          description
        end
      end

      class AnyArgMatcher < SingletonMatcher
        def ===(_other)
          true
        end

        def description
          "anything"
        end

        def inspect
          description
        end
      end

      class NoArgsMatcher < SingletonMatcher
        def description
          "no args"
        end

        def inspect
          description
        end
      end

      class BooleanMatcher < SingletonMatcher
        def ===(value)
          true == value || false == value
        end

        def description
          "boolean"
        end

        def inspect
          description
        end
      end

      class BaseHashMatcher
        def initialize(expected)
          @expected = expected
        end

        def ===(predicate, actual)
          @expected.__send__(predicate) do |key, value|
            actual.key?(key) && Rex::Matchers::FuzzyMatcher.values_match?(value, actual[key])
          end
        rescue NoMethodError
          false
        end

        def description(name)
          "#{name}(#{formatted_expected_hash.inspect.sub(/^\{/, '').sub(/\}$/, '')})"
        end

        def inspect
          description
        end

        private

        def formatted_expected_hash
          Hash[@expected.map { |key, value| [Rex::Matchers::ObjectFormatter.prepare(key), Rex::Matchers::ObjectFormatter.prepare(value)] }]
        end
      end

      class HashIncludingMatcher < BaseHashMatcher
        def ===(actual)
          super(:all?, actual)
        end

        def description
          super("hash_including")
        end
      end

      class HashExcludingMatcher < BaseHashMatcher
        def ===(actual)
          super(:none?, actual)
        end

        def description
          super("hash_not_including")
        end
      end

      class ArrayIncludingMatcher
        def initialize(expected)
          @expected = expected
        end

        def ===(actual)
          actual = actual.uniq
          @expected.uniq.all? do |expected_element|
            actual.any? { |actual_element| Rex::Matchers::FuzzyMatcher.values_match?(expected_element, actual_element) }
          end
        rescue NoMethodError
          false
        end

        def description
          "array_including(#{@expected.map { |item| Rex::Matchers::ObjectFormatter.format(item) }.join(', ')})"
        end

        def inspect
          description
        end
      end

      class ArrayExcludingMatcher
        def initialize(unexpected)
          @unexpected = unexpected.uniq
        end

        def ===(actual)
          actual = actual.uniq
          return true unless (actual & @unexpected).any?

          actual.none? do |actual_element|
            @unexpected.any? { |unexpected_element| Rex::Matchers::FuzzyMatcher.values_match?(unexpected_element, actual_element) }
          end
        rescue NoMethodError
          false
        end

        def description
          "array_excluding(#{@unexpected.map { |item| Rex::Matchers::ObjectFormatter.format(item) }.join(', ')})"
        end

        def inspect
          description
        end
      end

      class DuckTypeMatcher
        def initialize(*methods_to_respond_to)
          @methods_to_respond_to = methods_to_respond_to
        end

        def ===(value)
          @methods_to_respond_to.all? { |message| value.respond_to?(message) }
        end

        def description
          "duck_type(#{@methods_to_respond_to.map(&:inspect).join(', ')})"
        end

        def inspect
          description
        end
      end

      class InstanceOf
        def initialize(klass)
          @klass = klass
        end

        def ===(actual)
          actual.instance_of?(@klass)
        end

        def description
          "an_instance_of(#{@klass.name})"
        end

        def inspect
          description
        end
      end

      class KindOf
        def initialize(klass)
          @klass = klass
        end

        def ===(actual)
          @klass === actual
        end

        def description
          "kind_of(#{@klass.name})"
        end

        def inspect
          description
        end
      end
    end

    # Whether the arguments a message arrived with satisfy `with(...)`.
    class ArgumentListMatcher
      attr_reader :expected_args

      def initialize(*expected_args)
        @expected_args = expected_args
        ensure_expected_args_valid!
      end
      ruby2_keywords(:initialize)

      def args_match?(*actual_args)
        expected_args = resolve_expected_args_based_on(actual_args)
        return false if expected_args.size != actual_args.size

        Rex::Matchers::FuzzyMatcher.values_match?(expected_args, actual_args)
      end
      ruby2_keywords(:args_match?)

      def resolve_expected_args_based_on(actual_args)
        return [] if [ArgumentMatchers::NoArgsMatcher::INSTANCE] == expected_args

        any_args_index = expected_args.index { |arg| ArgumentMatchers::AnyArgsMatcher::INSTANCE.equal?(arg) }
        return expected_args unless any_args_index

        replace_any_args_with_splat_of_anything(any_args_index, actual_args.count)
      end

      private

      def replace_any_args_with_splat_of_anything(before_count, actual_args_count)
        any_args_count = actual_args_count - expected_args.count + 1
        after_count = expected_args.count - before_count - 1
        any_args = 1.upto(any_args_count).map { ArgumentMatchers::AnyArgMatcher::INSTANCE }
        expected_args.first(before_count) + any_args + expected_args.last(after_count)
      end

      def ensure_expected_args_valid!
        if expected_args.count { |arg| ArgumentMatchers::AnyArgsMatcher::INSTANCE.equal?(arg) } > 1
          raise ArgumentError, "`any_args` can only be passed to `with` once but you have passed it multiple times."
        elsif expected_args.count > 1 && expected_args.any? { |arg| ArgumentMatchers::NoArgsMatcher::INSTANCE.equal?(arg) }
          raise ArgumentError, "`no_args` can only be passed as a singleton argument to `with` (i.e. `with(no_args)`), but you have passed additional arguments."
        end
      end

      MATCH_ALL = new(ArgumentMatchers::AnyArgsMatcher::INSTANCE)
    end

    # One expectation or stub for one message: what to answer, and how many
    # times the message may and must arrive.
    class MessageExpectation
      attr_reader :message, :orig_object, :expected_from, :argument_list_matcher,
                  :actual_received_count, :expected_received_count, :error_generator
      attr_writer :expected_received_count, :expected_from, :argument_list_matcher
      protected :expected_received_count=, :expected_from=

      def initialize(error_generator, expected_from, method_double, type = :expectation, opts = {}, &implementation_block)
        @type = type
        @error_generator = error_generator
        @error_generator.opts = error_generator.opts.merge(opts)
        @expected_from = expected_from
        @method_double = method_double
        @orig_object = @method_double.object
        @message = @method_double.method_name
        @actual_received_count = 0
        @actual_received_count_write_mutex = Thread::Mutex.new
        @expected_received_count = type == :expectation ? 1 : :any
        @argument_list_matcher = ArgumentListMatcher::MATCH_ALL
        @order_group = Rex::Mocks::OrderGroup.instance
        @order_group.register(self) if type != :stub
        @expectation_type = type
        @ordered = false
        @at_least = @at_most = @exactly = nil
        @args_to_yield = []
        @failed_fast = nil
        @eval_context = nil
        @yield_receiver_to_implementation_block = false
        @similar_messages = []
        @implementation = Implementation.new
        self.inner_implementation_action = implementation_block
      end

      def and_yield_receiver_to_implementation
        @yield_receiver_to_implementation_block = true
        self
      end

      def and_return(first_value, *values, &_block)
        raise ArgumentError, "`and_return` is not supported with negative message expectations" if negative?

        values.unshift(first_value)
        @expected_received_count = [@expected_received_count, values.size].max unless ignoring_args? || (@expected_received_count == 0 && @at_least)
        self.terminal_implementation_action = AndReturnImplementation.new(values)
        nil
      end

      def and_call_original
        wrap_original(__method__) do |original, *args, &block|
          original.call(*args, &block)
        end
      end

      def and_wrap_original(&block)
        wrap_original(__method__, &block)
      end

      def and_raise(*args)
        raise_already_invoked_error_if_necessary(__method__)
        self.terminal_implementation_action = proc { raise(*args) }
        nil
      end

      def and_throw(*args)
        raise_already_invoked_error_if_necessary(__method__)
        self.terminal_implementation_action = proc { throw(*args) }
        nil
      end

      def and_yield(*args, &block)
        raise_already_invoked_error_if_necessary(__method__)
        yield @eval_context = Object.new if block

        @args_to_yield << args
        self.initial_implementation_action = AndYieldImplementation.new(@args_to_yield, @eval_context, @error_generator)
        self
      end

      def and_invoke(first_proc, *procs)
        all_procs = procs.unshift(first_proc)
        if all_procs.any? { |item| !item.respond_to?(:call) }
          raise ArgumentError, "Arguments to `and_invoke` must be callable."
        end

        @expected_received_count = [@expected_received_count, all_procs.size].max unless ignoring_args? || (@expected_received_count == 0 && @at_least)
        self.terminal_implementation_action = AndInvokeImplementation.new(all_procs)
        nil
      end

      def exactly(count, &block)
        raise_already_invoked_error_if_necessary(__method__)
        self.inner_implementation_action = block
        set_expected_received_count :exactly, count
        self
      end

      def at_least(count, &block)
        raise_already_invoked_error_if_necessary(__method__)
        set_expected_received_count :at_least, count
        if count == 0
          raise "at_least(0) has been removed, use allow(...).to receive(:message) instead"
        end

        self.inner_implementation_action = block
        self
      end

      def at_most(count, &block)
        raise_already_invoked_error_if_necessary(__method__)
        self.inner_implementation_action = block
        set_expected_received_count :at_most, count
        self
      end

      def times(&block)
        self.inner_implementation_action = block
        self
      end
      alias time times

      def never
        error_generator.raise_double_negation_error("expect(obj)") if negative?
        @expected_received_count = 0
        self
      end

      def once(&block)
        self.inner_implementation_action = block
        set_expected_received_count :exactly, 1
        self
      end

      def twice(&block)
        self.inner_implementation_action = block
        set_expected_received_count :exactly, 2
        self
      end

      def thrice(&block)
        self.inner_implementation_action = block
        set_expected_received_count :exactly, 3
        self
      end

      def with(*args, &block)
        raise_already_invoked_error_if_necessary(__method__)
        if args.empty?
          raise ArgumentError, "`with` must have at least one argument. Use `no_args` matcher to set the expectation of receiving no arguments."
        end

        self.inner_implementation_action = block
        @argument_list_matcher = ArgumentListMatcher.new(*args)
        self
      end
      ruby2_keywords(:with)

      def ordered(&block)
        if type == :stub
          raise "`ordered` is not supported with `allow` or `stub`"
        end

        self.inner_implementation_action = block
        additional_expected_calls.times { @order_group.register(self) }
        @ordered = true
        self
      end

      def ordered?
        @ordered
      end

      def type
        @expectation_type
      end

      def expected_args
        @argument_list_matcher.expected_args
      end

      def matches?(message, *args)
        @message == message && @argument_list_matcher.args_match?(*args)
      end
      ruby2_keywords(:matches?)

      def safe_invoke(parent_stub, *args, &block)
        invoke_incrementing_actual_calls_by(1, false, parent_stub, *args, &block)
      end
      ruby2_keywords(:safe_invoke)

      def invoke(parent_stub, *args, &block)
        invoke_incrementing_actual_calls_by(1, true, parent_stub, *args, &block)
      end
      ruby2_keywords(:invoke)

      def invoke_without_incrementing_received_count(parent_stub, *args, &block)
        invoke_incrementing_actual_calls_by(0, true, parent_stub, *args, &block)
      end
      ruby2_keywords(:invoke_without_incrementing_received_count)

      def negative?
        @expected_received_count == 0 && !@at_least
      end

      def called_max_times?
        @expected_received_count != :any &&
          !@at_least &&
          @expected_received_count > 0 &&
          @actual_received_count >= @expected_received_count
      end

      def matches_name_but_not_args(message, *args)
        @message == message && !@argument_list_matcher.args_match?(*args)
      end
      ruby2_keywords(:matches_name_but_not_args)

      def verify_messages_received
        return if expected_messages_received?

        generate_error
      end

      def expected_messages_received?
        ignoring_args? || matches_exact_count? || matches_at_least_count? || matches_at_most_count?
      end

      def ensure_expected_ordering_received!
        @order_group.verify_invocation_order(self) if @ordered
        true
      end

      def ignoring_args?
        @expected_received_count == :any
      end

      def matches_at_least_count?
        @at_least && @actual_received_count >= @expected_received_count
      end

      def matches_at_most_count?
        @at_most && @actual_received_count <= @expected_received_count
      end

      def matches_exact_count?
        @expected_received_count == @actual_received_count
      end

      def similar_messages
        @similar_messages ||= []
      end

      def advise(*args)
        similar_messages << args
      end
      ruby2_keywords(:advise)

      def unadvise(args)
        similar_messages.delete_if { |message| args.include?(message) }
      end

      def generate_error
        if similar_messages.empty?
          @error_generator.raise_expectation_error(
            @message, @expected_received_count, @argument_list_matcher,
            @actual_received_count, expectation_count_type, expected_args,
            @expected_from
          )
        else
          @error_generator.raise_similar_message_args_error(self, @similar_messages, @expected_from)
        end
      end

      def raise_unexpected_message_args_error(args_for_multiple_calls)
        @error_generator.raise_unexpected_message_args_error(self, args_for_multiple_calls)
      end

      def expectation_count_type
        return :at_least if @at_least
        return :at_most if @at_most

        nil
      end

      def description_for(verb)
        @error_generator.describe_expectation(verb, @message, @expected_received_count, @actual_received_count, expected_args)
      end

      def raise_out_of_order_error
        @error_generator.raise_out_of_order_error @message
      end

      def additional_expected_calls
        return 0 if @expectation_type == :stub || !@exactly

        @expected_received_count - 1
      end

      def increase_actual_received_count!
        @actual_received_count_write_mutex.synchronize { @actual_received_count += 1 }
      end

      def actual_received_count_matters?
        @at_least || @at_most || @exactly
      end

      private

      def invoke_incrementing_actual_calls_by(increment, allowed_to_fail, parent_stub, *args, &block)
        args.unshift(orig_object) if yield_receiver_to_implementation_block?

        if negative? || (allowed_to_fail && (@exactly || @at_most) && (@actual_received_count == @expected_received_count))
          @actual_received_count_write_mutex.synchronize { @actual_received_count += increment }
          @failed_fast = true
          @error_generator.raise_expectation_error(
            @message, @expected_received_count,
            @argument_list_matcher,
            @actual_received_count, expectation_count_type,
            args, nil
          )
        end

        @order_group.handle_order_constraint self

        if implementation.present?
          implementation.call(*args, &block)
        elsif parent_stub
          parent_stub.invoke(nil, *args, &block)
        end
      ensure
        @actual_received_count_write_mutex.synchronize { @actual_received_count += increment } unless @failed_fast
      end
      ruby2_keywords(:invoke_incrementing_actual_calls_by)

      def yield_receiver_to_implementation_block?
        @yield_receiver_to_implementation_block
      end

      def implementation
        @implementation
      end

      def has_been_invoked?
        @actual_received_count > 0
      end

      def raise_already_invoked_error_if_necessary(calling_customization)
        return unless has_been_invoked?

        raise "The message expectation for #{orig_object.inspect}.#{message} has already been invoked " \
              "and cannot be modified further (e.g. using `#{calling_customization}`). All message expectation " \
              "customizations must be applied before it is used for the first time."
      end

      def set_expected_received_count(relativity, n)
        raise "`count` is not supported with negative message expectations" if negative?

        @at_least = (relativity == :at_least)
        @at_most = (relativity == :at_most)
        @exactly = (relativity == :exactly)
        @expected_received_count = case n
                                   when Numeric then n
                                   when :once then 1
                                   when :twice then 2
                                   when :thrice then 3
                                   end
      end

      def initial_implementation_action=(action)
        implementation.initial_action = action
      end

      def inner_implementation_action=(action)
        return unless action

        warn_about_stub_override if implementation.inner_action
        implementation.inner_action = action
      end

      def terminal_implementation_action=(action)
        implementation.terminal_action = action
      end

      def warn_about_stub_override
        Rex.warn_with "WARNING: An expectation of `:#{@message}` was set on `#{@error_generator.intro}`, " \
                      "but has already been set up with a block implementation.", call_site: nil
      end

      def wrap_original(method_name, &block)
        if TestDouble === @method_double.object
          @error_generator.raise_only_valid_on_a_partial_double(method_name)
        else
          warn_about_stub_override if implementation.inner_action
          @implementation = AndWrapOriginalImplementation.new(@method_double.original_implementation_callable, block)
          @yield_receiver_to_implementation_block = false
        end
        nil
      end
    end

    # What a stub or an expectation does when the message arrives: an
    # optional yield first, then the block given or the configured answer.
    class Implementation
      attr_accessor :initial_action, :inner_action, :terminal_action

      def call(*args, &block)
        actions.map do |action|
          action.call(*args, &block)
        end.last
      end
      ruby2_keywords(:call)

      def present?
        actions.any?
      end

      private

      def actions
        [initial_action, inner_action, terminal_action].compact
      end
    end

    class AndReturnImplementation
      def initialize(values_to_return)
        @values_to_return = values_to_return
      end

      def call(*_args_to_ignore, &_block)
        if @values_to_return.size > 1
          @values_to_return.shift
        else
          @values_to_return.first
        end
      end
    end

    class AndInvokeImplementation
      def initialize(procs_to_invoke)
        @procs_to_invoke = procs_to_invoke
      end

      def call(*args, &block)
        proc = if @procs_to_invoke.size > 1
                 @procs_to_invoke.shift
               else
                 @procs_to_invoke.first
               end
        proc.call(*args, &block)
      end
      ruby2_keywords(:call)
    end

    class AndYieldImplementation
      def initialize(args_to_yield, eval_context, error_generator)
        @args_to_yield = args_to_yield
        @eval_context = eval_context
        @error_generator = error_generator
      end

      def call(*_args_to_ignore, &block)
        return @error_generator.raise_missing_block_error(@args_to_yield) unless block

        value = nil
        block_signature = Rex::Core::MethodSignature.new(block)
        @args_to_yield.each do |args|
          unless block_signature.accepts?(args.size)
            @error_generator.raise_wrong_arity_error(args, block_signature)
          end
          value = @eval_context ? @eval_context.instance_exec(*args, &block) : yield(*args)
        end
        value
      end
    end

    class AndWrapOriginalImplementation
      def initialize(method, block)
        @method = method
        @block = block
      end

      def initial_action=(_value)
        raise "`and_wrap_original` is a terminal action and cannot be combined with `and_yield`"
      end

      def inner_action=(_value)
        raise "`and_wrap_original` is a terminal action and cannot be combined with a block implementation"
      end

      def terminal_action=(_value)
        raise "`and_wrap_original` is a terminal action and cannot be combined with `and_return`"
      end

      def present?
        true
      end

      def inner_action
        true
      end

      def call(*args, &block)
        @block.call(@method, *args, &block)
      end
      ruby2_keywords(:call)
    end

    # The order `ordered` expectations must arrive in.
    class OrderGroup
      def self.instance
        Mocks.space.instance_variable_get(:@order_group) || Mocks.space.instance_variable_set(:@order_group, new)
      end

      def initialize
        @expectations = []
        @invocation_order = []
        @index = 0
      end

      def register(expectation)
        @expectations << expectation
      end

      def invoked(message)
        @invocation_order << message
      end

      def ready_for?(expectation)
        remaining_expectations.find(&:ordered?) == expectation
      end

      def consume
        remaining_expectations.each_with_index do |expectation, index|
          next unless expectation.ordered?

          @index += index + 1
          return expectation
        end
        nil
      end

      def handle_order_constraint(expectation)
        return unless expectation.ordered? && remaining_expectations.include?(expectation)
        return consume if ready_for?(expectation)

        expectation.raise_out_of_order_error
      end

      def verify_invocation_order(expectation)
        expectation.raise_out_of_order_error unless expectations_invoked_in_order?
        true
      end

      def clear
        @index = 0
        @invocation_order.clear
        @expectations.clear
      end

      def empty?
        @expectations.empty?
      end

      private

      def remaining_expectations
        @expectations[@index..-1] || []
      end

      def expectations_invoked_in_order?
        invoked_expectations == expected_invocations
      end

      def invoked_expectations
        @expectations.select { |expectation| expectation.ordered? && @invocation_order.include?(expectation) }
      end

      def expected_invocations
        @invocation_order.map { |invocation| expectation_for(invocation) }.compact
      end

      def expectation_for(message)
        @expectations.find { |expectation| message == expectation }
      end
    end
  end
end

require "rex/mocks/proxy"
require "rex/mocks/verifying"
require "rex/mocks/targets"
require "rex/mocks/any_instance"
require "rex/mocks/mutate_const"
require "rex/mocks/configuration"

Rex::Mocks.configuration.reset_syntaxes_to_default
