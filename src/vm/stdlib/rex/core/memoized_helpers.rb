require "rex/support/reentrant_mutex"

module Rex
  module Core
    module MemoizedHelpers
      def subject
        __memoized.fetch_or_store(:subject) do
          described = described_class || self.class.metadata.fetch(:description_args).first
          Class === described ? described.new : described
        end
      end

      def should(matcher=nil, message=nil)
        enforce_value_expectation(matcher, 'should')
        Rex::Expectations::PositiveExpectationHandler.handle_matcher(subject, matcher, message)
      end

      def should_not(matcher=nil, message=nil)
        enforce_value_expectation(matcher, 'should_not')
        Rex::Expectations::NegativeExpectationHandler.handle_matcher(subject, matcher, message)
      end

      def is_expected
        expect(subject)
      end

      attr_reader :__memoized
      private :__memoized

    private

      def initialize(*)
        __init_memoized
        super
      end

      def __init_memoized
        @__memoized = if Rex.configuration.threadsafe?
                        ThreadsafeMemoized.new
                      else
                        NonThreadSafeMemoized.new
                      end
      end

      def enforce_value_expectation(matcher, method_name)
        return if matcher_supports_value_expectations?(matcher)

        Rex.deprecate(
          "#{method_name} #{Rex::Support::ObjectFormatter.format(matcher)}",
          :message =>
            "The implicit block expectation syntax is deprecated, you should pass " \
            "a block to `expect` to use the provided block expectation matcher " \
            "(#{Rex::Support::ObjectFormatter.format(matcher)}), " \
            "or the matcher must implement `supports_value_expectations?`."
        )
      end

      def matcher_supports_value_expectations?(matcher)
        matcher.supports_value_expectations?
      rescue
        true
      end

      class ThreadsafeMemoized
        def initialize
          @memoized = {}
          @mutex = Support::ReentrantMutex.new
        end

        def fetch_or_store(key)
          @memoized.fetch(key) do
            @mutex.synchronize do
              @memoized.fetch(key) { @memoized[key] = yield }
            end
          end
        end
      end

      class NonThreadSafeMemoized
        def initialize
          @memoized = {}
        end

        def fetch_or_store(key)
          @memoized.fetch(key) { @memoized[key] = yield }
        end
      end

      class ContextHookMemoized
        def self.isolate_for_context_hook(example_group_instance)
          exploding_memoized = self

          example_group_instance.instance_exec do
            @__memoized = exploding_memoized

            begin
              yield
            ensure
              __init_memoized
            end
          end
        end

        def self.fetch_or_store(key, &_block)
          description = if key == :subject
                          "subject"
                        else
                          "let declaration `#{key}`"
                        end

          raise <<-EOS
#{description} accessed in #{article} #{hook_expression} hook at:
  #{CallerFilter.first_non_rspec_line}

`let` and `subject` declarations are not intended to be called
in #{article} #{hook_expression} hook, as they exist to define state that
is reset between each example, while #{hook_expression} exists to
#{hook_intention}.
EOS
        end

        class Before < self
          def self.hook_expression
            "`before(:context)`"
          end

          def self.article
            "a"
          end

          def self.hook_intention
            "define state that is shared across examples in an example group"
          end
        end

        class After < self
          def self.hook_expression
            "`after(:context)`"
          end

          def self.article
            "an"
          end

          def self.hook_intention
            "cleanup state that is shared across examples in an example group"
          end
        end
      end

      module ClassMethods
        def let(name, &block)
          raise "#let or #subject called without a block" if block.nil?

          if [:initialize, :to_s].include?(name.to_sym)
            raise ArgumentError, "#let or #subject called with reserved name `#{name}`"
          end

          our_module = MemoizedHelpers.module_for(self)

          if our_module.instance_methods(false).include?(name)
            our_module.__send__(:remove_method, name)
          end
          our_module.__send__(:define_method, name, &block)

          if instance_methods(false).include?(name)
            remove_method(name)
          end

          if block.arity == 1
            define_method(name) { __memoized.fetch_or_store(name) { super(Rex.current_example, &nil) } }
          else
            define_method(name) { __memoized.fetch_or_store(name) { super(&nil) } }
          end
        end

        def let!(name, &block)
          let(name, &block)
          before { __send__(name) }
        end

        def subject(name=nil, &block)
          if name
            let(name, &block)
            alias_method :subject, name

            self::NamedSubjectPreventSuper.__send__(:define_method, name) do
              raise NotImplementedError, "`super` in named subjects is not supported"
            end
          else
            let(:subject, &block)
          end
        end

        def subject!(name=nil, &block)
          subject(name, &block)
          before { subject }
        end
      end

      def self.module_for(example_group)
        get_constant_or_yield(example_group, :LetDefinitions) do
          mod = Module.new do
            include(Module.new {
              example_group.const_set(:NamedSubjectPreventSuper, self)
            })
          end

          example_group.const_set(:LetDefinitions, mod)
          mod
        end
      end

      def self.define_helpers_on(example_group)
        example_group.__send__(:include, module_for(example_group))
      end

      if Module.method(:const_defined?).arity == 1
        def self.get_constant_or_yield(example_group, name)
          if example_group.const_defined?(name)
            example_group.const_get(name)
          else
            yield
          end
        end
      else
        def self.get_constant_or_yield(example_group, name)
          if example_group.const_defined?(name, (check_ancestors = false))
            example_group.const_get(name, check_ancestors)
          else
            yield
          end
        end
      end
    end
  end
end
