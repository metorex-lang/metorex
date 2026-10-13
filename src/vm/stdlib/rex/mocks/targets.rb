module Rex
  module Mocks
    # The methods an example calls to make doubles and set up messages.
    module ExampleMethods
      include ArgumentMatchers

      def double(*args)
        ExampleMethods.declare_double(Double, *args)
      end

      def instance_double(doubled_class, *args)
        ref = ObjectReference.for(doubled_class)
        ExampleMethods.declare_verifying_double(InstanceVerifyingDouble, ref, *args)
      end

      def class_double(doubled_class, *args)
        ref = ObjectReference.for(doubled_class)
        ExampleMethods.declare_verifying_double(ClassVerifyingDouble, ref, *args)
      end

      def object_double(object_or_name, *args)
        ref = ObjectReference.for(object_or_name, :allow_direct_object_refs)
        ExampleMethods.declare_verifying_double(ObjectVerifyingDouble, ref, *args)
      end

      def spy(*args)
        double(*args).as_null_object
      end

      def instance_spy(*args)
        instance_double(*args).as_null_object
      end

      def object_spy(*args)
        object_double(*args).as_null_object
      end

      def class_spy(*args)
        class_double(*args).as_null_object
      end

      def allow(target)
        AllowanceTarget.new(target)
      end

      def allow_any_instance_of(klass)
        AnyInstanceAllowanceTarget.new(klass)
      end

      def expect_any_instance_of(klass)
        AnyInstanceExpectationTarget.new(klass)
      end

      def receive(method_name, &block)
        Matchers::Receive.new(method_name, block)
      end

      def receive_messages(message_return_value_hash)
        Matchers::ReceiveMessages.new(message_return_value_hash)
      end

      def receive_message_chain(*messages, &block)
        Matchers::ReceiveMessageChain.new(messages, &block)
      end

      def have_received(method_name, &block)
        Matchers::HaveReceived.new(method_name, &block)
      end

      def hide_const(constant_name)
        ConstantMutator.hide(constant_name)
      end

      def stub_const(constant_name, value, options = {})
        ConstantMutator.stub(constant_name, value, options)
      end

      def self.declare_verifying_double(type, ref, *args)
        if Mocks.configuration.verify_doubled_constant_names && !ref.defined?
          ErrorGenerator.new.raise_verifying_double_not_defined_error(ref)
        end

        declare_double(type, ref, *args)
      end

      def self.declare_double(type, *args)
        args << {} unless Hash === args.last
        type.new(*args)
      end
    end

    # `allow(x)`: a stub that need not be called.
    class TargetBase
      def initialize(target)
        @target = target
      end
    end

    class AllowanceTarget < TargetBase
      def to(matcher, &block)
        unless matcher.respond_to?(:setup_allowance)
          raise UnsupportedMatcherError,
                "only the `receive`, `have_received` and `receive_messages` matchers are supported " \
                "with `allow(...).to`, but you have provided: #{matcher}"
        end
        matcher.setup_allowance(@target, &block)
      end

      def not_to(matcher, *_args)
        raise NegationUnsupportedError,
              "`allow(...).not_to #{matcher.matcher_name}` is not supported since it " \
              "doesn't really make sense. What would it even mean?"
      end
      alias to_not not_to
    end

    class AnyInstanceAllowanceTarget < TargetBase
      def to(matcher, &block)
        matcher.setup_any_instance_allowance(@target, &block)
      end

      def not_to(matcher, *_args)
        raise NegationUnsupportedError,
              "`allow_any_instance_of(...).not_to #{matcher.matcher_name}` is not supported since it " \
              "doesn't really make sense. What would it even mean?"
      end
      alias to_not not_to
    end

    class AnyInstanceExpectationTarget < TargetBase
      def to(matcher, &block)
        matcher.setup_any_instance_expectation(@target, &block)
      end

      def not_to(matcher, &block)
        matcher.setup_any_instance_negative_expectation(@target, &block)
      end
      alias to_not not_to
    end

    module Matchers
      # The chained calls on a `receive` matcher, replayed onto the stub or
      # expectation it sets up.
      class Receive
        attr_reader :message

        def initialize(message, block)
          @message = message
          @block = block
          @recorded_customizations = []
        end

        def matcher_name
          "receive"
        end

        def description
          describable.description_for("receive")
        end

        def setup_expectation(subject, &block)
          warn_if_any_instance("expect", subject)
          @describable = setup_mock_proxy_method_substitute(subject, :add_message_expectation, block)
        end
        alias matches? setup_expectation

        # `never` goes first, so a customization such as `and_return` that a
        # negative expectation refuses is refused.
        def setup_negative_expectation(subject, &block)
          @recorded_customizations.unshift [:never, [], nil]
          setup_expectation(subject, &block)
        end
        alias does_not_match? setup_negative_expectation

        def setup_allowance(subject, &block)
          warn_if_any_instance("allow", subject)
          setup_mock_proxy_method_substitute(subject, :add_stub, block)
        end

        def setup_any_instance_expectation(subject, &block)
          setup_any_instance_method_substitute(subject, :should_receive, block)
        end

        def setup_any_instance_negative_expectation(subject, &block)
          setup_any_instance_method_substitute(subject, :should_not_receive, block)
        end

        def setup_any_instance_allowance(subject, &block)
          setup_any_instance_method_substitute(subject, :stub, block)
        end

        %i[with and_return and_raise and_throw and_yield and_call_original and_wrap_original and_invoke
           once twice thrice exactly times time never at_least at_most ordered].each do |method|
          define_method(method) do |*args, &block|
            @recorded_customizations << [method, args, block]
            self
          end
          ruby2_keywords(method)
        end

        private

        def describable
          @describable ||= DefaultDescribable.new(@message)
        end

        def warn_if_any_instance(_expression, _subject)
        end

        def setup_mock_proxy_method_substitute(subject, method, block)
          proxy = Mocks.space.proxy_for(subject)
          setup_method_substitute(proxy, method, block, subject)
        end

        def setup_any_instance_method_substitute(subject, method, block)
          proxy = Mocks.space.any_instance_recorder_for(subject)
          setup_method_substitute(proxy, method, block)
        end

        def setup_method_substitute(host, method, block, *_subject)
          block = move_block_to_last_customization(block)
          expectation = host.__send__(method, @message.to_sym, &(@block || block))
          @recorded_customizations.each do |customization_method, customization_args, customization_block|
            expectation.__send__(customization_method, *customization_args, &customization_block)
          end
          expectation
        end

        def move_block_to_last_customization(block)
          last = @recorded_customizations.last
          return block unless last

          last_method, last_args, last_block = last
          if last_block.nil? && block
            @recorded_customizations[-1] = [last_method, last_args, block]
            nil
          else
            block
          end
        end

        DefaultDescribable = Struct.new(:message) do
          def description_for(verb)
            "#{verb} #{message}"
          end
        end
      end

      class ReceiveMessages
        def initialize(message_return_value_hash)
          @message_return_value_hash = message_return_value_hash
          @backtrace_line = Mocks.caller_line
        end

        def matcher_name
          "receive_messages"
        end

        def description
          "receive messages: #{@message_return_value_hash.inspect}"
        end

        def setup_expectation(subject)
          warn_about_block if block_given?
          each_message_on(Mocks.space.proxy_for(subject)) do |host, message, return_value|
            host.add_simple_expectation(message, return_value, @backtrace_line)
          end
        end
        alias matches? setup_expectation

        def setup_negative_expectation(_subject)
          raise NegationUnsupportedError,
                "`expect(...).to_not receive_messages` is not supported since it " \
                "doesn't really make sense. What would it even mean?"
        end
        alias does_not_match? setup_negative_expectation

        def setup_allowance(subject)
          warn_about_block if block_given?
          each_message_on(Mocks.space.proxy_for(subject)) do |host, message, return_value|
            host.add_simple_stub(message, return_value)
          end
        end

        def setup_any_instance_expectation(subject)
          each_message_on(Mocks.space.any_instance_recorder_for(subject)) do |recorder, message, return_value|
            recorder.should_receive(message).and_return(return_value)
          end
        end

        def setup_any_instance_allowance(subject)
          Mocks.space.any_instance_recorder_for(subject).stub(@message_return_value_hash)
        end

        private

        def each_message_on(host)
          @message_return_value_hash.each do |message, value|
            yield host, message, value
          end
        end

        def warn_about_block
          raise "Implementation blocks aren't supported with `receive_messages`"
        end
      end

      class ReceiveMessageChain
        def initialize(chain, &block)
          @chain = chain
          @block = block
          @recorded_customizations = []
        end

        %i[and_return and_throw and_raise and_yield and_call_original with].each do |method|
          define_method(method) do |*args, &block|
            @recorded_customizations << [method, args, block]
            self
          end
          ruby2_keywords(method)
        end

        def matcher_name
          "receive_message_chain"
        end

        def description
          "receive message chain #{formatted_chain}"
        end

        def setup_allowance(subject, &block)
          chain = chain_on(subject, *@chain, &(@block || block))
          replay_customizations(chain)
        end

        def setup_any_instance_allowance(subject, &block)
          proxy = Mocks.space.any_instance_recorder_for(subject)
          chain = proxy.stub_chain(*@chain, &(@block || block))
          replay_customizations(chain)
        end

        def setup_expectation(subject, &block)
          chain = chain_on(subject, *@chain, expect_last: true, &(@block || block))
          replay_customizations(chain)
        end
        alias matches? setup_expectation

        def setup_negative_expectation(*_args)
          raise NegationUnsupportedError,
                "`expect(...).not_to receive_message_chain` is not supported " \
                "since it doesn't really make sense. What would it even mean?"
        end
        alias does_not_match? setup_negative_expectation

        private

        def chain_on(subject, *chain, expect_last: false, &block)
          chain = chain.flat_map { |link| link.to_s.split(".").map(&:to_sym) }
          first, *rest = chain
          proxy = Mocks.space.proxy_for(subject)
          if rest.empty?
            return expect_last ? proxy.add_message_expectation(first, &block) : proxy.add_stub(first, &block)
          end

          existing = proxy.method_double_for(first).stubs.first
          next_object = if existing&.respond_to?(:__chain_target, true)
                          existing.__send__(:__chain_target)
                        else
                          link = Double.new
                          stub = proxy.add_stub(first) { link }
                          stub.define_singleton_method(:__chain_target) { link }
                          link
                        end
          chain_on(next_object, *rest, expect_last: expect_last, &block)
        end

        def replay_customizations(chain)
          @recorded_customizations.each do |method, args, block|
            chain.__send__(method, *args, &block)
          end
        end

        def formatted_chain
          @formatted_chain ||= @chain.map { |link| link.is_a?(Hash) ? link.keys.first : link }.join(".")
        end
      end

      # `expect(spy).to have_received(:message)`.
      class HaveReceived
        COUNT_CONSTRAINTS = %w[exactly at_least at_most times time once twice thrice].freeze
        ARGS_CONSTRAINTS = %w[with].freeze
        CONSTRAINTS = COUNT_CONSTRAINTS + ARGS_CONSTRAINTS + %w[ordered]

        def initialize(method_name, &block)
          @method_name = method_name
          @block = block
          @constraints = []
          @subject = nil
        end

        def matcher_name
          "have_received"
        end

        def matches?(subject, &block)
          @block ||= block
          @subject = subject
          @expectation = expect
          mock_proxy.ensure_implemented(@method_name)

          expected_messages_received_in_order?
        end

        def does_not_match?(subject)
          @subject = subject
          ensure_count_unconstrained
          @expectation = expect.never
          mock_proxy.ensure_implemented(@method_name)
          expected_messages_received_in_order?
        end

        def failure_message
          capture_failure_message
        end

        def failure_message_when_negated
          capture_failure_message
        end

        def description
          (@expectation ||= expect).description_for("have received")
        end

        def setup_expectation(subject, &block)
          notify_failure_message unless matches?(subject, &block)
        end

        def setup_negative_expectation(subject, &block)
          notify_failure_message unless does_not_match?(subject, &block)
        end

        def setup_allowance(_subject, &_block)
          disallow("allow", " as it would have no effect")
        end

        def setup_any_instance_allowance(_subject, &_block)
          disallow("allow_any_instance_of")
        end

        def setup_any_instance_expectation(_subject, &_block)
          disallow("expect_any_instance_of")
        end

        def setup_using_proxy(_subject, &_block)
          disallow("expect(...).to receive_message_chain")
        end

        CONSTRAINTS.each do |expectation|
          define_method expectation do |*args|
            @constraints << [expectation, *args]
            self
          end
          ruby2_keywords(expectation)
        end

        private

        def disallow(type, reason = "")
          raise MockExpectationError,
                "Using #{type}(...) with the `have_received` " \
                "matcher is not supported#{reason}."
        end

        def expect
          expectation = mock_proxy.build_expectation(@method_name)
          apply_constraints_to expectation
          expectation
        end

        def apply_constraints_to(expectation)
          @constraints.each do |constraint|
            expectation.__send__(*constraint)
          end
        end

        def ensure_count_unconstrained
          return unless count_constraint

          raise ArgumentError, "can't use #{count_constraint} when negative"
        end

        def count_constraint
          @constraints.map(&:first).find do |constraint|
            COUNT_CONSTRAINTS.include?(constraint)
          end
        end

        def capture_failure_message
          notify_failure_message
        rescue MockExpectationError => error
          error.message
        end

        def notify_failure_message
          mock_proxy.check_for_unexpected_arguments(@expectation)
          @expectation.generate_error
        end

        def expected_messages_received_in_order?
          mock_proxy.replay_received_message_on @expectation, &@block
          @expectation.expected_messages_received? && @expectation.ensure_expected_ordering_received!
        end

        def mock_proxy
          Mocks.space.proxy_for(@subject)
        end
      end
    end

    # `allow_any_instance_of` and `expect_any_instance_of`: a stub on the
    # class that hands each instance its own proxy the first time it is
    # called.
    module AnyInstance
      class Recorder
        attr_reader :klass

        def initialize(klass)
          @klass = klass
          @stubbed_methods = {}
          @expectations = []
          @instances_seen = {}
        end

        def stub(method_name_or_hash, &block)
          if Hash === method_name_or_hash
            method_name_or_hash.each do |name, value|
              stub(name).and_return(value)
            end
            return
          end

          chain = Chain.new(self, :add_stub, method_name_or_hash.to_sym, block)
          observe!(method_name_or_hash.to_sym)
          (@stubbed_methods[method_name_or_hash.to_sym] ||= []) << chain
          chain
        end

        def stub_chain(*chain, &block)
          raise NotImplementedError, "receive_message_chain on any instance is not supported"
        end

        def should_receive(method_name, &block)
          chain = Chain.new(self, :add_message_expectation, method_name.to_sym, block)
          chain.expectation_fulfilled = false
          observe!(method_name.to_sym)
          (@stubbed_methods[method_name.to_sym] ||= []) << chain
          @expectations << chain
          chain
        end

        def should_not_receive(method_name, &block)
          should_receive(method_name, &block).never
        end

        def playback!(instance, method_name)
          return if @instances_seen.key?([instance.__id__, method_name])

          @instances_seen[[instance.__id__, method_name]] = instance
          (@stubbed_methods[method_name] || []).each do |chain|
            chain.playback!(instance)
          end
        end

        def verify
          missing = @expectations.reject(&:expectation_fulfilled?).map(&:method_name)
          return if missing.empty?

          if missing.size == 1
            message = "Exactly one instance should have received the following message(s) but didn't: #{missing.join(', ')}"
          else
            message = "Exactly one instance should have received the following message(s) but didn't: #{missing.sort.join(', ')}"
          end
          error = MockExpectationError.new(message)
          location = @expectations.find { |chain| !chain.expectation_fulfilled? }&.expected_from
          error.set_backtrace([location].compact + caller)
          raise error
        end

        def reset
          @stubbed_methods.each_key do |method_name|
            restore_original(method_name)
          end
          @stubbed_methods.clear
          @expectations.clear
        end

        private

        def observe!(method_name)
          return if @originals&.key?(method_name)

          @originals ||= {}
          owned = @klass.instance_methods(false).include?(method_name) ||
                  @klass.private_instance_methods(false).include?(method_name)
          @originals[method_name] = owned ? @klass.instance_method(method_name) : :none
          recorder = self
          @klass.__send__(:define_method, method_name) do |*args, &block|
            recorder.playback!(self, method_name)
            __send__(method_name, *args, &block)
          end
          @klass.__send__(:ruby2_keywords, method_name)
        end

        def restore_original(method_name)
          original = @originals&.delete(method_name)
          return unless original

          @klass.__send__(:remove_method, method_name)
          @klass.__send__(:define_method, method_name, original) unless original == :none
        end
      end

      # The customizations of one any-instance stub or expectation, replayed
      # onto each instance that receives the message.
      class Chain
        attr_reader :method_name, :expected_from
        attr_writer :expectation_fulfilled

        def initialize(recorder, proxy_method, method_name, block)
          @recorder = recorder
          @proxy_method = proxy_method
          @method_name = method_name
          @block = block
          @customizations = []
          @expected_from = Mocks.caller_line
          @expectation_fulfilled = true
        end

        def expectation_fulfilled?
          @expectation_fulfilled
        end

        def playback!(instance)
          proxy = Mocks.space.proxy_for(instance)
          expectation = proxy.__send__(@proxy_method, @method_name, expected_from: @expected_from, &@block)
          @customizations.each do |method, args, block|
            expectation.__send__(method, *args, &block)
          end
          if @proxy_method == :add_message_expectation
            chain = self
            expectation.define_singleton_method(:invoke) do |*invoke_args, &invoke_block|
              chain.expectation_fulfilled = true
              super(*invoke_args, &invoke_block)
            end
          end
          expectation
        end

        %i[with and_return and_raise and_throw and_yield and_call_original
           once twice thrice exactly times time never at_least at_most].each do |method|
          define_method(method) do |*args, &block|
            @customizations << [method, args, block]
            @expectation_fulfilled = true if method == :never
            self
          end
          ruby2_keywords(method)
        end
      end
    end

    # `stub_const` and `hide_const`, undone when the example ends.
    module ConstantMutator
      def self.stub(constant_name, value, _options = {})
        parent, name = split(constant_name)
        existed = parent.const_defined?(name, false)
        original = parent.const_get(name, false) if existed
        Mocks.space.instance_variable_get(:@constant_resets)&.push(nil)
        register_reset do
          parent.__send__(:remove_const, name) if parent.const_defined?(name, false)
          parent.const_set(name, original) if existed
        end
        parent.__send__(:remove_const, name) if existed
        parent.const_set(name, value)
        value
      end

      def self.hide(constant_name)
        parent, name = split(constant_name)
        return unless parent.const_defined?(name, false)

        original = parent.const_get(name, false)
        register_reset { parent.const_set(name, original) unless parent.const_defined?(name, false) }
        parent.__send__(:remove_const, name)
        nil
      end

      def self.split(constant_name)
        names = constant_name.to_s.sub(/\A::/, "").split("::")
        name = names.pop
        parent = names.inject(Object) { |scope, part| scope.const_get(part) }
        [parent, name.to_sym]
      end

      def self.register_reset(&block)
        resets = Mocks.space.instance_variable_get(:@constant_resets) || Mocks.space.instance_variable_set(:@constant_resets, [])
        resets.compact!
        resets << block
      end
    end

    class Space
      alias reset_all_without_constants reset_all

      def reset_all
        reset_all_without_constants
      ensure
        (@constant_resets || []).compact.reverse_each(&:call)
        @constant_resets = []
      end
    end

    # `should_receive` and its relatives on every object, on when a suite
    # asks for the old syntax.
    module Syntax
      module ShouldMethods
        def should_receive(message, opts = {}, &block)
          Mocks.expect_message(self, message, opts, &block)
        end

        def should_not_receive(message, &block)
          Mocks.expect_message(self, message, {}, &block).never
        end

        def stub(message_or_hash, opts = {}, &block)
          if Hash === message_or_hash
            message_or_hash.each { |message, value| Mocks.allow_message(self, message).and_return(value) }
          else
            Mocks.allow_message(self, message_or_hash, opts, &block)
          end
        end

        def unstub(message)
          Mocks.space.proxy_for(self).remove_stub(message)
        end

        def stub_chain(*chain, &block)
          Matchers::ReceiveMessageChain.new(chain, &block).setup_allowance(self)
        end

        def as_null_object
          Mocks.space.proxy_for(self).as_null_object
        end

        def null_object?
          Mocks.space.proxy_for(self).null_object?
        end

        def received_message?(message, *args, &block)
          Mocks.space.proxy_for(self).received_message?(message, *args, &block)
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
      end
    end
  end

  module Expectations
    class ExpectationTarget
      alias rex_to_without_mocks to
      alias rex_not_to_without_mocks not_to

      # A `receive` or `have_received` matcher sets up its message rather
      # than matching a value.
      def to(matcher = nil, message = nil, &block)
        if matcher.respond_to?(:setup_expectation)
          return matcher.setup_expectation(@target, &block)
        end

        rex_to_without_mocks(matcher, message, &block)
      end

      def not_to(matcher = nil, message = nil, &block)
        if matcher.respond_to?(:setup_negative_expectation)
          return matcher.setup_negative_expectation(@target, &block)
        end

        rex_not_to_without_mocks(matcher, message, &block)
      end
      alias to_not not_to
    end
  end
end
