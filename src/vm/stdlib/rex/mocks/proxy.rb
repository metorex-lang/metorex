module Rex
  module Mocks
    # Receives every message sent to an object that has stubs or
    # expectations, and answers it from them.
    class Proxy
      attr_reader :object

      def initialize(object, error_generator = ErrorGenerator.new(object))
        @object = object
        @error_generator = error_generator
        @method_doubles = Hash.new { |hash, name| hash[name] = MethodDouble.new(@object, name, self) }
        @messages_received = []
        @messages_received_mutex = Thread::Mutex.new
        @null_object = false
      end

      def ensure_can_be_proxied!(object)
        return unless object.is_a?(Symbol) || object.frozen?
        return if object.nil?

        message = "Cannot proxy frozen objects"
        message += ", rspec-mocks relies on proxies for method stubbing and expectations." unless object.is_a?(Symbol)
        raise ArgumentError, message
      end

      def as_null_object
        @null_object = true
        @object
      end

      def null_object?
        @null_object
      end

      def original_method_handle_for(message)
        nil
      end

      def method_double_if_exists_for_message(message)
        method_double_for(message) if @method_doubles.key?(message.to_sym)
      end

      def ensure_implemented(*_args)
      end

      def add_message_expectation(method_name, opts = {}, &block)
        location = opts.fetch(:expected_from) { Mocks.caller_line }
        meth_double = method_double_for(method_name)
        meth_double.add_expectation @error_generator, location, opts, &block
      end

      def add_simple_expectation(method_name, response, location)
        method_double_for(method_name).add_simple_expectation method_name, response, @error_generator, location
      end

      def build_expectation(method_name)
        meth_double = method_double_for(method_name)
        meth_double.build_expectation(@error_generator)
      end

      def replay_received_message_on(expectation, &block)
        expected_method_name = expectation.message
        meth_double = method_double_for(expected_method_name)

        if meth_double.expectations.any?
          @error_generator.raise_expectation_on_mocked_method(expected_method_name)
        end

        unless null_object? || meth_double.stubs.any?
          @error_generator.raise_expectation_on_unstubbed_method(expected_method_name)
        end

        @messages_received_mutex.synchronize do
          @messages_received.each do |(actual_method_name, args, received_block)|
            next unless expectation.matches?(actual_method_name, *args)

            expectation.safe_invoke(nil)
            block&.call(*args, &received_block)
          end
        end
      end

      def check_for_unexpected_arguments(expectation)
        @messages_received_mutex.synchronize do
          return if @messages_received.empty?
          return if @messages_received.any? { |method_name, args, _| expectation.matches?(method_name, *args) }

          name_but_not_args, others = @messages_received.partition do |(method_name, args, _)|
            expectation.matches_name_but_not_args(method_name, *args)
          end
          return if name_but_not_args.empty? && !others.empty?

          expectation.raise_unexpected_message_args_error(name_but_not_args.map { |args| args[1] })
        end
      end

      def add_stub(method_name, opts = {}, &implementation)
        location = opts.fetch(:expected_from) { Mocks.caller_line }
        method_double_for(method_name).add_stub @error_generator, location, opts, &implementation
      end

      def add_simple_stub(method_name, response)
        method_double_for(method_name).add_simple_stub method_name, response
      end

      def remove_stub(method_name)
        method_double_for(method_name).remove_stub
      end

      def remove_stub_if_present(method_name)
        method_double_for(method_name).remove_stub_if_present
      end

      def verify
        @method_doubles.each_value(&:verify)
      end

      def reset
        @messages_received_mutex.synchronize { @messages_received.clear }
        @method_doubles.each_value(&:reset)
      end

      def received_message?(method_name, *args, &block)
        @messages_received_mutex.synchronize do
          @messages_received.any? { |array| array == [method_name, args, block] }
        end
      end

      def messages_arg_list
        @messages_received_mutex.synchronize { @messages_received.map { |_, args, _| args } }
      end

      def has_negative_expectation?(message)
        method_double_for(message).expectations.find(&:negative?)
      end

      def record_message_received(message, *args, &block)
        @order_group&.invoked SpecificMessage.new(object, message, args)
        @messages_received_mutex.synchronize { @messages_received << [message, args, block] }
      end
      ruby2_keywords(:record_message_received)

      def message_received(message, *args, &block)
        record_message_received message, *args, &block

        expectation = find_matching_expectation(message, *args)
        stub = find_matching_method_stub(message, *args)

        if (stub && expectation && expectation.called_max_times?) || (stub && !expectation)
          expectation.increase_actual_received_count! if expectation && expectation.actual_received_count_matters?
          if (expectation = find_almost_matching_expectation(message, *args))
            expectation.advise(*args) unless expectation.expected_messages_received?
          end
          stub.invoke(nil, *args, &block)
        elsif expectation
          expectation.unadvise(messages_arg_list)
          expectation.invoke(stub, *args, &block)
        elsif (expectation = find_almost_matching_expectation(message, *args))
          expectation.advise(*args) if null_object? unless expectation.expected_messages_received?

          if null_object? || !has_negative_expectation?(message)
            expectation.raise_unexpected_message_args_error([args])
          end
        elsif (stub = find_almost_matching_stub(message, *args))
          stub.advise(*args)
          raise_missing_default_stub_error(stub, [args])
        elsif Class === @object
          @object.superclass.__send__(message, *args, &block)
        else
          @object.__send__(:method_missing, message, *args, &block)
        end
      end
      ruby2_keywords(:message_received)

      def raise_unexpected_message_error(method_name, args)
        @error_generator.raise_unexpected_message_error method_name, args
      end

      def raise_missing_default_stub_error(expectation, args_for_multiple_calls)
        @error_generator.raise_missing_default_stub_error(expectation, args_for_multiple_calls)
      end

      def visibility_for(_method_name)
        "public"
      end

      def prepended_modules_of_singleton_class
        []
      end

      def method_double_for(message)
        @method_doubles[message.to_sym]
      end

      private

      def find_matching_expectation(method_name, *args)
        find_best_matching_expectation_for(method_name) do |expectation|
          expectation.matches?(method_name, *args)
        end
      end
      ruby2_keywords(:find_matching_expectation)

      def find_almost_matching_expectation(method_name, *args)
        find_best_matching_expectation_for(method_name) do |expectation|
          expectation.matches_name_but_not_args(method_name, *args)
        end
      end
      ruby2_keywords(:find_almost_matching_expectation)

      def find_best_matching_expectation_for(method_name)
        first_match = nil

        method_double_for(method_name).expectations.each do |expectation|
          next unless yield expectation
          return expectation unless expectation.called_max_times?

          first_match ||= expectation
        end

        first_match
      end

      def find_matching_method_stub(method_name, *args)
        method_double_for(method_name).stubs.find { |stub| stub.matches?(method_name, *args) }
      end
      ruby2_keywords(:find_matching_method_stub)

      def find_almost_matching_stub(method_name, *args)
        method_double_for(method_name).stubs.find { |stub| stub.matches_name_but_not_args(method_name, *args) }
      end
      ruby2_keywords(:find_almost_matching_stub)
    end

    SpecificMessage = Struct.new(:object, :message, :args) do
      def ==(expectation)
        expectation.orig_object == object && expectation.matches?(message, *args)
      end
    end

    class TestDoubleProxy < Proxy
      def reset
        @method_doubles.clear
        object.__disallow_further_usage!
        super
      end
    end

    # The proxy of a real object: a stub or an expectation replaces one of
    # its methods until the example ends, when the original comes back.
    class PartialDoubleProxy < Proxy
      def original_method_handle_for(message)
        if Rex::Core::ExampleGroup === object
          nil
        else
          ::Kernel.instance_method(:method).bind(object).call(message)
        end
      rescue NameError
        nil
      end

      def add_simple_expectation(method_name, response, location)
        method_double_for(method_name).configure_method
        super
      end

      def add_simple_stub(method_name, response)
        method_double_for(method_name).configure_method
        super
      end

      def visibility_for(method_name)
        object_singleton_class = class << @object; self; end
        if object_singleton_class.private_method_defined?(method_name)
          "private"
        elsif object_singleton_class.protected_method_defined?(method_name)
          "protected"
        else
          "public"
        end
      end

      def reset
        @method_doubles.each_value(&:reset)
        super
      end

      def message_received(message, *args, &block)
        RubyFeatures.ensure_not_a_ruby_object_cycle(message)
        super
      end
      ruby2_keywords(:message_received)
    end

    module RubyFeatures
      def self.ensure_not_a_ruby_object_cycle(_message)
      end
    end

    # A partial double that refuses a method the object does not have, and a
    # call with arguments the method would not take.
    class VerifyingPartialDoubleProxy < PartialDoubleProxy
      def initialize(object, error_generator = ErrorGenerator.new(object))
        super
        @doubled_module = DirectObjectReference.new(object)
        @method_doubles = Hash.new do |hash, name|
          hash[name] = VerifyingExistingMethodDouble.for(object, name, self)
        end
      end

      def ensure_implemented(method_name)
        return if Mocks.configuration.temporarily_suppress_partial_double_verification
        return if @object.respond_to?(method_name, true)

        @error_generator.raise_unimplemented_error(@doubled_module, method_name, @object)
      end

      def add_stub(method_name, opts = {}, &implementation)
        ensure_implemented(method_name)
        super
      end

      def add_message_expectation(method_name, opts = {}, &block)
        ensure_implemented(method_name)
        super
      end

      def method_reference
        @method_doubles
      end
    end

    # The stubs and expectations of one method of one object, and the
    # singleton method that sends calls to them.
    class MethodDouble
      attr_reader :method_name, :object, :expectations, :stubs, :method_stasher

      def initialize(object, method_name, proxy)
        @method_name = method_name
        @object = object
        @proxy = proxy
        @original_visibility = nil
        @method_stasher = InstanceMethodStasher.new(object, method_name)
        @method_is_proxied = false
        @expectations = []
        @stubs = []
      end

      def original_implementation_callable
        @original_implementation_callable ||= original_method || proc do |*args, &block|
          @object.__send__(:method_missing, @method_name, *args, &block)
        end
      end

      def original_method
        @original_method ||= @method_stasher.original_method || @proxy.original_method_handle_for(method_name)
      end

      def visibility
        @proxy.visibility_for(@method_name)
      end

      def object_singleton_class
        class << @object; self; end
      end

      def configure_method
        @original_visibility = visibility
        @method_stasher.stash unless @method_is_proxied
        define_proxy_method
      end

      def define_proxy_method
        return if @method_is_proxied

        save_original_implementation_callable!
        double = self
        name = @method_name
        visibility_now = visibility
        object_singleton_class.class_exec do
          define_method(name) do |*args, &block|
            double.proxy_method_invoked(self, *args, &block)
          end
          ruby2_keywords(name)
          __send__(visibility_now, name)
        end
        @method_is_proxied = true
      rescue RuntimeError, TypeError => error
        raise error unless error.message.include?("frozen") || error.message.include?("can't define singleton")

        raise ArgumentError, "Cannot proxy frozen objects, rspec-mocks relies on proxies for method stubbing and expectations."
      end

      def proxy_method_invoked(_obj, *args, &block)
        @proxy.message_received method_name, *args, &block
      end
      ruby2_keywords(:proxy_method_invoked)

      def restore_original_method
        return unless @method_is_proxied

        remove_method_from_definition_target
        @method_stasher.restore if @method_stasher.method_is_stashed?
        restore_original_visibility
        @method_is_proxied = false
      rescue NameError
        @method_is_proxied = false
      end

      def show_frozen_warning
        Rex.warn_with "WARNING: rspec-mocks was unable to restore the original `#{@method_name}` " \
                      "method on #{@object.inspect} because it has been frozen.  If you reuse this " \
                      "object, `#{@method_name}` will continue to respond with its stub implementation.",
                      call_site: nil, use_spec_location_as_call_site: true
      end

      def restore_original_visibility
        return unless @original_visibility && object_singleton_class.method_defined?(@method_name)

        object_singleton_class.__send__(@original_visibility, method_name)
      end

      def verify
        expectations.each(&:verify_messages_received)
      end

      def reset
        restore_original_method
        clear
      end

      def clear
        expectations.clear
        stubs.clear
      end

      def message_expectation_class
        MessageExpectation
      end

      def add_expectation(error_generator, expected_from, opts, &implementation)
        configure_method
        expectation = message_expectation_class.new(error_generator, expected_from, self, :expectation, opts, &implementation)
        expectations << expectation
        expectation
      end

      # An expectation `have_received` builds after the fact names no line of
      # its own, so its failure points at the `expect` that checked it.
      def build_expectation(error_generator)
        message_expectation_class.new(error_generator, nil, self)
      end

      def add_stub(error_generator, expected_from, opts = {}, &implementation)
        configure_method
        stub = message_expectation_class.new(error_generator, expected_from, self, :stub, opts, &implementation)
        stubs.unshift stub
        stub
      end

      def add_simple_stub(method_name, response)
        setup_simple_method_double method_name, response, stubs
      end

      def add_simple_expectation(method_name, response, error_generator, backtrace_line)
        setup_simple_method_double method_name, response, expectations, error_generator, backtrace_line
      end

      def setup_simple_method_double(method_name, response, collection, error_generator = nil, backtrace_line = nil)
        define_proxy_method
        me = SimpleMessageExpectation.new(method_name, response, error_generator, backtrace_line)
        collection.unshift me
        me
      end

      def add_default_stub(*args, &implementation)
        return if stubs.any?

        add_stub(*args, &implementation)
      end

      def remove_stub
        raise_method_not_stubbed_error if stubs.empty?
        remove_stub_if_present
      end

      def remove_stub_if_present
        expectations.empty? ? reset : stubs.clear
      end

      def raise_method_not_stubbed_error
        raise MockExpectationError, "The method `#{method_name}` was not stubbed or was already unstubbed"
      end

      def remove_method_from_definition_target
        object_singleton_class.__send__(:remove_method, @method_name)
      end

      private

      def save_original_implementation_callable!
        original_implementation_callable
      end
    end

    # A stub written as `double(name: value)` or `receive_messages`: it
    # answers its value and records nothing more.
    class SimpleMessageExpectation
      def initialize(message, response, error_generator, backtrace_line = nil)
        @message = message
        @response = response
        @error_generator = error_generator
        @backtrace_line = backtrace_line
        @received = false
      end

      def invoke(*_)
        @received = true
        @response
      end

      def matches?(message, *_)
        @message == message
      end

      def called_max_times?
        false
      end

      def verify_messages_received
        return if @received

        @error_generator.raise_expectation_error(
          @message, 1, ArgumentListMatcher::MATCH_ALL, 0, nil, [], @backtrace_line
        )
      end

      def unadvise(_)
      end

      def negative?
        false
      end

      def matches_name_but_not_args(*)
        false
      end

      def actual_received_count_matters?
        false
      end
    end

    # Keeps the singleton class's own method a stub replaces, so the stub
    # can be taken away and the method put back exactly.
    class InstanceMethodStasher
      def initialize(object, method)
        @object = object
        @method = method
        @klass = (class << object; self; end)
        @original_method = nil
        @method_is_stashed = false
      end

      attr_reader :original_method

      def method_is_stashed?
        @method_is_stashed
      end

      def stash
        return unless method_defined_directly_on_klass?

        @original_method ||= ::Kernel.instance_method(:method).bind(@object).call(@method)
        @stashed_unbound = @klass.instance_method(@method)
        @method_is_stashed = true
      end

      def restore
        return unless @method_is_stashed

        @klass.__send__(:remove_method, @method) if defined_directly?
        @klass.__send__(:define_method, @method, @stashed_unbound)
        @method_is_stashed = false
      end

      private

      def defined_directly?
        @klass.instance_methods(false).include?(@method) ||
          @klass.private_instance_methods(false).include?(@method) ||
          @klass.protected_instance_methods(false).include?(@method)
      end

      def method_defined_directly_on_klass?
        method_defined_on_klass? && method_owned_by_klass?
      end

      def method_defined_on_klass?(klass = @klass)
        klass.method_defined?(@method) || klass.private_method_defined?(@method) || klass.protected_method_defined?(@method)
      end

      def method_owned_by_klass?
        owner = @klass.instance_method(@method).owner
        owner == @klass
      end
    end
  end
end
