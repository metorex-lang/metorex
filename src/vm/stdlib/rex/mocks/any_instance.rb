module Rex
  module Mocks
    # `allow_any_instance_of`, `expect_any_instance_of`, and the `:should`
    # syntax's `any_instance`: stubs and expectations recorded against a
    # class and played back onto each instance that receives the message.
    module AnyInstance
      # One stub or expectation, with the customizations chained onto it.
      class Chain
        attr_reader :expected_from

        def initialize(recorder, *args, &block)
          @recorder = recorder
          @expectation_args = args
          @expectation_block = block
          @argument_list_matcher = ArgumentListMatcher::MATCH_ALL
          @expected_from = Mocks.caller_line
        end

        # The customizations a chain records to play back later.
        module Customizations
          def self.record(method_name)
            define_method(method_name) do |*args, &block|
              record(method_name, *args, &block)
            end
            ruby2_keywords(method_name)
          end

          record :and_return
          record :and_raise
          record :and_throw
          record :and_yield
          record :and_call_original
          record :and_wrap_original
          record :with
          record :once
          record :twice
          record :thrice
          record :exactly
          record :times
          record :time
          record :never
          record :at_least
          record :at_most
        end

        include Customizations

        def playback!(instance)
          message_expectation = create_message_expectation_on(instance)
          messages.inject(message_expectation) do |object, message|
            object.__send__(*message.first, &message.last)
          end
        end

        def constrained_to_any_of?(*constraints)
          constraints.any? do |constraint|
            messages.any? do |message|
              message.first.first == constraint
            end
          end
        end

        def matches_args?(*args)
          @argument_list_matcher.args_match?(*args)
        end

        def expectation_fulfilled!
          @expectation_fulfilled = true
        end

        def never
          AnyInstance.error_generator.raise_double_negation_error("expect_any_instance_of(MyClass)") if negated?
          super
        end

        def with(*args, &block)
          @argument_list_matcher = ArgumentListMatcher.new(*args)
          super
        end
        ruby2_keywords(:with)

        private

        def negated?
          messages.any? { |(message, *_), _| message == :never }
        end

        def messages
          @messages ||= []
        end

        def last_message
          messages.last.first.first unless messages.empty?
        end

        def record(rspec_method_name, *args, &block)
          verify_invocation_order(rspec_method_name, *args, &block)
          messages << [args.unshift(rspec_method_name), block]
          self
        end
        ruby2_keywords(:record)
      end

      class ErrorGenerator < ::Rex::Mocks::ErrorGenerator
        def raise_second_instance_received_message_error(unfulfilled_expectations, backtrace_line = nil)
          __raise "Exactly one instance should have received the following " \
                  "message(s) but didn't: #{unfulfilled_expectations.sort.join(', ')}", backtrace_line
        end

        def raise_does_not_implement_error(klass, method_name)
          __raise "#{klass} does not implement ##{method_name}"
        end

        def raise_message_already_received_by_other_instance_error(method_name, object_inspect, invoked_instance)
          __raise "The message '#{method_name}' was received by #{object_inspect} " \
                  "but has already been received by #{invoked_instance}"
        end

        def raise_not_supported_with_prepend_error(method_name, problem_mod)
          __raise "Using `any_instance` to stub a method (#{method_name}) that has been " \
                  "defined on a prepended module (#{problem_mod}) is not supported."
        end

        def raise_method_not_stubbed_error(method_name)
          __raise "The method `#{method_name}` was not stubbed or was already unstubbed"
        end
      end

      def self.error_generator
        @error_generator ||= ErrorGenerator.new
      end

      class ExpectationChain < Chain
        def expectation_fulfilled?
          @expectation_fulfilled || constrained_to_any_of?(:never)
        end

        def initialize(*args, &block)
          @expectation_fulfilled = false
          super
        end

        private

        def verify_invocation_order(_rspec_method_name, *_args, &_block)
        end
      end

      class PositiveExpectationChain < ExpectationChain
        ExpectationInvocationOrder = {
          and_return: [:with, nil],
          and_raise: [:with, nil]
        }.freeze

        private

        def create_message_expectation_on(instance)
          proxy = ::Rex::Mocks.space.proxy_for(instance)
          method_name, opts = @expectation_args
          opts = (opts || {}).merge(expected_from: expected_from)

          expectation = proxy.add_message_expectation(method_name, opts, &@expectation_block)
          if ::Rex::Mocks.configuration.yield_receiver_to_any_instance_implementation_blocks?
            expectation.and_yield_receiver_to_implementation
          end

          expectation
        end

        def invocation_order
          ExpectationInvocationOrder
        end
      end

      class StubChain < Chain
        InvocationOrder = {
          and_return: [:with, nil],
          and_raise: [:with, nil],
          and_yield: [:with, :and_yield, nil],
          and_throw: [:with, nil],
          and_call_original: [:with, nil],
          and_wrap_original: [:with, nil]
        }.freeze

        EmptyInvocationOrder = {}.freeze

        def expectation_fulfilled?
          true
        end

        private

        def create_message_expectation_on(instance)
          proxy = ::Rex::Mocks.space.proxy_for(instance)
          method_name, opts = @expectation_args
          opts = (opts || {}).merge(expected_from: expected_from)

          stub = proxy.add_stub(method_name, opts, &@expectation_block)
          @recorder.stubs[stub.message] << stub

          if ::Rex::Mocks.configuration.yield_receiver_to_any_instance_implementation_blocks?
            stub.and_yield_receiver_to_implementation
          end

          stub
        end

        def invocation_order
          InvocationOrder
        end

        def verify_invocation_order(rspec_method_name, *_args, &_block)
          return if invocation_order.fetch(rspec_method_name, [nil]).include?(last_message)

          raise NoMethodError, "Undefined method #{rspec_method_name}"
        end
      end

      class StubChainChain < StubChain
        def initialize(*args)
          super
          @expectation_fulfilled = false
        end

        private

        def create_message_expectation_on(instance)
          ::Rex::Mocks::StubChain.stub_chain_on(instance, *@expectation_args, &@expectation_block)
        end

        def invocation_order
          EmptyInvocationOrder
        end
      end

      class ExpectChainChain < StubChain
        def initialize(*args)
          super
          @expectation_fulfilled = false
        end

        def expectation_fulfilled?
          @expectation_fulfilled
        end

        def playback!(instance)
          super.tap { @expectation_fulfilled = true }
        end

        private

        def create_message_expectation_on(instance)
          ::Rex::Mocks::ExpectChain.expect_chain_on(instance, *@expectation_args, &@expectation_block)
        end

        def invocation_order
          EmptyInvocationOrder
        end
      end

      # The chains recorded for each method of one class.
      class MessageChains
        def initialize
          @chains_by_method_name = Hash.new { |hash, key| hash[key] = [] }
        end

        def [](method_name)
          @chains_by_method_name[method_name]
        end

        def add(method_name, chain)
          @chains_by_method_name[method_name] << chain
          chain
        end

        def remove_stub_chains_for!(method_name)
          @chains_by_method_name[method_name].reject! do |chain|
            StubChain === chain
          end
        end

        def has_expectation?(method_name)
          @chains_by_method_name[method_name].find do |chain|
            ExpectationChain === chain
          end
        end

        def each_unfulfilled_expectation_matching(method_name, *args)
          @chains_by_method_name[method_name].each do |chain|
            yield chain if !chain.expectation_fulfilled? && chain.matches_args?(*args)
          end
        end

        def all_expectations_fulfilled?
          @chains_by_method_name.all? do |_method_name, chains|
            chains.all?(&:expectation_fulfilled?)
          end
        end

        def unfulfilled_expectations
          @chains_by_method_name.map do |method_name, chains|
            method_name.to_s if ExpectationChain === chains.last && !chains.last.expectation_fulfilled?
          end.compact
        end

        # Where the first expectation no instance received was written.
        def unfulfilled_expectation_location
          @chains_by_method_name.each_value do |chains|
            chain = chains.last
            return chain.expected_from if ExpectationChain === chain && !chain.expectation_fulfilled?
          end
          nil
        end

        def received_expected_message!(method_name)
          @chains_by_method_name[method_name].each(&:expectation_fulfilled!)
        end

        def playback!(instance, method_name)
          raise_if_second_instance_to_receive_message(instance)
          @chains_by_method_name[method_name].each do |chain|
            chain.playback!(instance)
          end
        end

        private

        def raise_if_second_instance_to_receive_message(instance)
          @instance_with_expectation ||= instance if ExpectationChain === instance
          return unless ExpectationChain === instance
          return if @instance_with_expectation.equal?(instance)

          AnyInstance.error_generator.raise_second_instance_received_message_error(unfulfilled_expectations)
        end
      end

      # The chains recorded for one class, and the methods it wraps to play
      # them back.
      class Recorder
        attr_reader :message_chains, :stubs, :klass

        def initialize(klass)
          @message_chains = MessageChains.new
          @stubs = Hash.new { |hash, key| hash[key] = [] }
          @observed_methods = []
          @played_methods = {}
          @backed_up_method_owner = {}
          @klass = klass
          @expectation_set = false

          return unless ::Rex::Mocks.configuration.verify_partial_doubles?

          ::Rex::Mocks.configuration.verifying_double_callbacks.each do |block|
            block.call(ObjectReference.for(klass))
          end
        end

        def stub(method_name, &block)
          observe!(method_name)
          message_chains.add(method_name, StubChain.new(self, method_name, &block))
        end

        def stub_chain(*method_names_and_optional_return_values, &block)
          normalize_chain(*method_names_and_optional_return_values) do |method_name, args|
            observe!(method_name)
            message_chains.add(method_name, StubChainChain.new(self, *args, &block))
          end
        end

        def expect_chain(*method_names_and_optional_return_values, &block)
          @expectation_set = true
          normalize_chain(*method_names_and_optional_return_values) do |method_name, args|
            observe!(method_name)
            message_chains.add(method_name, ExpectChainChain.new(self, *args, &block))
          end
        end

        def should_receive(method_name, &block)
          @expectation_set = true
          observe!(method_name)
          message_chains.add(method_name, PositiveExpectationChain.new(self, method_name, &block))
        end

        def should_not_receive(method_name, &block)
          should_receive(method_name, &block).never
        end

        def unstub(method_name)
          unless @observed_methods.include?(method_name.to_sym)
            AnyInstance.error_generator.raise_method_not_stubbed_error(method_name)
          end
          message_chains.remove_stub_chains_for!(method_name)
          stubs[method_name].clear
          stop_observing!(method_name) unless message_chains.has_expectation?(method_name)
        end

        def verify
          return unless @expectation_set
          return if message_chains.all_expectations_fulfilled?

          AnyInstance.error_generator.raise_second_instance_received_message_error(
            message_chains.unfulfilled_expectations,
            message_chains.unfulfilled_expectation_location
          )
        end

        def stop_all_observation!
          @observed_methods.each { |method_name| restore_method!(method_name) }
        end

        def playback!(instance, method_name)
          ::Rex::Mocks.space.ensure_registered(instance)
          message_chains.playback!(instance, method_name)
          @played_methods[method_name] = instance
          received_expected_message!(method_name) if message_chains.has_expectation?(method_name)
        end

        def instance_that_received(method_name)
          @played_methods[method_name]
        end

        def build_alias_method_name(method_name)
          "__#{method_name}_without_any_instance__"
        end

        def already_observing?(method_name)
          @observed_methods.include?(method_name) || super_class_observing?(method_name)
        end

        def notify_received_message(_object, message, args, _block)
          has_expectation = false

          message_chains.each_unfulfilled_expectation_matching(message, *args) do |expectation|
            has_expectation = true
            expectation.expectation_fulfilled!
          end

          return unless has_expectation

          restore_method!(message)
          mark_invoked!(message)
        end

        protected

        def stop_observing!(method_name)
          restore_method!(method_name)
          @observed_methods.delete(method_name)
          super_class_observers_for(method_name).each do |ancestor|
            ::Rex::Mocks.space.any_instance_recorder_for(ancestor).stop_observing!(method_name)
          end
        end

        private

        def ancestor_is_an_observer?(ancestor, method_name)
          return if ancestor == @klass

          ::Rex::Mocks.space.any_instance_recorder_for(ancestor).already_observing?(method_name)
        end

        def super_class_observers_for(method_name)
          @klass.ancestors.select do |ancestor|
            ancestor_is_an_observer?(ancestor, method_name)
          end
        end

        def super_class_observing?(method_name)
          @klass.ancestors.any? do |ancestor|
            ancestor_is_an_observer?(ancestor, method_name)
          end
        end

        def normalize_chain(*args)
          args.shift.to_s.split(".").map(&:to_sym).reverse.each { |name| args.unshift name }
          yield args.first, args
        end

        def received_expected_message!(method_name)
          message_chains.received_expected_message!(method_name)
          restore_method!(method_name)
          mark_invoked!(method_name)
        end

        def restore_method!(method_name)
          if public_protected_or_private_method_defined?(build_alias_method_name(method_name))
            restore_original_method!(method_name)
          else
            remove_dummy_method!(method_name)
          end
        end

        def restore_original_method!(method_name)
          return unless @klass.instance_method(method_name).owner == @klass

          alias_method_name = build_alias_method_name(method_name)
          @klass.class_exec(@backed_up_method_owner) do |backed_up_method_owner|
            remove_method method_name

            # A method defined on a superclass is reached again by removing
            # the copy; only one the class owned is aliased back.
            alias_method method_name, alias_method_name if backed_up_method_owner[method_name.to_sym] == self

            remove_method alias_method_name
          end
        end

        def remove_dummy_method!(method_name)
          @klass.class_exec do
            remove_method method_name
          end
        end

        def backup_method!(method_name)
          return unless public_protected_or_private_method_defined?(method_name)

          alias_method_name = build_alias_method_name(method_name)
          @backed_up_method_owner[method_name.to_sym] ||= @klass.instance_method(method_name).owner
          @klass.class_exec do
            alias_method alias_method_name, method_name
          end
        end

        def public_protected_or_private_method_defined?(method_name)
          MethodReference.method_defined_at_any_visibility?(@klass, method_name)
        end

        def observe!(method_name)
          allow_no_prepended_module_definition_of(method_name)

          if ::Rex::Mocks.configuration.verify_partial_doubles? &&
             !::Rex::Mocks.configuration.temporarily_suppress_partial_double_verification
            unless public_protected_or_private_method_defined?(method_name)
              AnyInstance.error_generator.raise_does_not_implement_error(@klass, method_name)
            end
          end

          stop_observing!(method_name) if already_observing?(method_name)
          @observed_methods << method_name
          backup_method!(method_name)
          recorder = self
          method_was_private = @klass.private_method_defined?(method_name)
          @klass.__send__(:define_method, method_name) do |*args, &block|
            recorder.playback!(self, method_name)
            __send__(method_name, *args, &block)
          end
          @klass.__send__(:private, method_name) if method_was_private
          @klass.__send__(:ruby2_keywords, method_name) if @klass.respond_to?(:ruby2_keywords, true)
        end

        def mark_invoked!(method_name)
          backup_method!(method_name)
          recorder = self
          @klass.__send__(:define_method, method_name) do |*_args, &_block|
            invoked_instance = recorder.instance_that_received(method_name)
            inspect = "#<#{self.class}:#{object_id} #{instance_variables.map { |name| "#{name}=#{instance_variable_get name}" }.join(', ')}>"
            AnyInstance.error_generator.raise_message_already_received_by_other_instance_error(
              method_name, inspect, invoked_instance
            )
          end
        end

        def allow_no_prepended_module_definition_of(method_name)
          prepended_modules = AnyInstance.prepended_modules_of(@klass)
          problem_mod = prepended_modules.find { |mod| mod.method_defined?(method_name) }
          return unless problem_mod

          AnyInstance.error_generator.raise_not_supported_with_prepend_error(method_name, problem_mod)
        end
      end

      # The modules prepended to `klass`, which sit ahead of it in its
      # ancestors.
      def self.prepended_modules_of(klass)
        ancestors = klass.ancestors
        singleton_index = ancestors.index(klass) || 0
        ancestors[0, singleton_index]
      end

      # What `Klass.any_instance` answers: each call goes to the recorder and
      # to the proxies of the instances already doubled.
      class Proxy
        def initialize(recorder, target_proxies)
          @recorder = recorder
          @target_proxies = target_proxies
        end

        def klass
          @recorder.klass
        end

        def stub(method_name_or_method_map, &block)
          if Hash === method_name_or_method_map
            method_name_or_method_map.each do |method_name, return_value|
              stub(method_name).and_return(return_value)
            end
          else
            perform_proxying(__method__, [method_name_or_method_map], block) do |proxy|
              proxy.add_stub(method_name_or_method_map, &block)
            end
          end
        end

        def unstub(method_name)
          perform_proxying(__method__, [method_name], nil) do |proxy|
            proxy.remove_stub_if_present(method_name)
          end
        end

        def stub_chain(*chain, &block)
          perform_proxying(__method__, chain, block) do |proxy|
            Mocks::StubChain.stub_chain_on(proxy.object, *chain, &block)
          end
        end

        def expect_chain(*chain, &block)
          perform_proxying(__method__, chain, block) do |proxy|
            Mocks::ExpectChain.expect_chain_on(proxy.object, *chain, &block)
          end
        end

        def should_receive(method_name, &block)
          perform_proxying(__method__, [method_name], block) do |proxy|
            # Yes, a stub, not a message expectation: one instance has to
            # receive the message, which the recorder checks.
            proxy.add_stub(method_name, &block)
          end
        end

        def should_not_receive(method_name, &block)
          perform_proxying(__method__, [method_name], block) do |proxy|
            proxy.add_message_expectation(method_name, &block).never
          end
        end

        private

        def perform_proxying(method_name, args, block, &target_proxy_block)
          recorder_value = @recorder.__send__(method_name, *args, &block)
          proxy_values = @target_proxies.map(&target_proxy_block)
          FluentInterfaceProxy.new([recorder_value] + proxy_values)
        end
      end

      # Hands each customization chained onto it to every target.
      class FluentInterfaceProxy < BasicObject
        def initialize(targets)
          @targets = targets
        end

        def respond_to_missing?(method_name, include_private = false)
          super || @targets.first.respond_to?(method_name, include_private)
        end

        def method_missing(*args, &block)
          return_values = @targets.map { |target| target.__send__(*args, &block) }
          FluentInterfaceProxy.new(return_values)
        end
        ruby2_keywords(:method_missing)
      end
    end

    # `stub_chain`, and `receive_message_chain` on an allowance.
    module StubChain
      def self.stub_chain_on(object, *chain, &block)
        Matchers::ReceiveMessageChain.new(chain).__send__(:chain_on, object, *chain, &block)
      end
    end

    # `receive_message_chain` on an expectation.
    module ExpectChain
      def self.expect_chain_on(object, *chain, &block)
        Matchers::ReceiveMessageChain.new(chain).__send__(:chain_on, object, *chain, expect_last: true, &block)
      end
    end
  end
end
