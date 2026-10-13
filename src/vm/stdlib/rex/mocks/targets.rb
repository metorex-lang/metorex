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
        if Mocks.configuration.verify_doubled_constant_names? && !ref.defined?
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
          proxy = Mocks.space.any_instance_proxy_for(subject)
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
          each_message_on(Mocks.space.any_instance_proxy_for(subject)) do |host, message, return_value|
            host.should_receive(message).and_return(return_value)
          end
        end

        def setup_any_instance_allowance(subject)
          Mocks.space.any_instance_proxy_for(subject).stub(@message_return_value_hash)
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
          proxy = Mocks.space.any_instance_proxy_for(subject)
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
    # `should_receive` and its relatives on every object, on when a suite
    # asks for the old syntax.
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
