module Rex
  module Matchers
    # `Rex::Matchers.define :name do |expected| ... end`.
    module DSL
      def define(name, &declarations)
        warn_about_block_args(name, declarations)
        define_method(name) do |*expected, &block_arg|
          Rex::Matchers::DSL::Matcher.new(name, declarations, self, *expected, &block_arg)
        end
      end
      alias matcher define

      private

      def warn_about_block_args(name, declarations)
        declarations.parameters.each do |type, argument_name|
          next unless type == :block

          Rex.warning("Your `#{name}` custom matcher receives a block argument (`#{argument_name}`), " \
                      "but due to limitations in ruby, RSpec cannot provide the block. Instead, " \
                      "use the `block_arg` method to access the block")
        end
      end

      # The class-level calls a `define` block makes.
      module Macros
        def match(options = {}, &match_block)
          define_user_override(:matches?, match_block) do |actual|
            @actual = actual
            begin
              super(*actual_arg_for(match_block))
            rescue Rex::ExpectationNotMet
              false
            end
          end
        end

        def match_unless_raises(expected_exception = Exception, &match_block)
          define_user_override(:matches?, match_block) do |actual|
            @actual = actual
            begin
              super(*actual_arg_for(match_block))
            rescue expected_exception => error
              @rescued_exception = error
              false
            else
              true
            end
          end
        end

        def match_when_negated(options = {}, &match_block)
          define_user_override(:does_not_match?, match_block) do |actual|
            begin
              @actual = actual
              super(*actual_arg_for(match_block))
            rescue Rex::ExpectationNotMet
              false
            end
          end
        end

        def failure_message(&definition)
          define_user_override(__method__, definition)
        end

        def failure_message_when_negated(&definition)
          define_user_override(__method__, definition)
        end

        def description(&definition)
          define_user_override(__method__, definition)
        end

        def diffable
          define_method(:diffable?) { true }
        end

        def supports_block_expectations
          define_method(:supports_block_expectations?) { true }
        end

        def chain(method_name, *attr_names, &definition)
          unless block_given? ^ attr_names.any?
            raise ArgumentError, "You must pass either a block or some attribute names (but not both) to `chain`."
          end

          definition = assign_attributes(attr_names) if attr_names.any?
          define_user_override(method_name, definition) do |*args, &block|
            super(*args, &block)
            @chained_method_clauses.push([method_name, args])
            self
          end
        end

        private

        def assign_attributes(attr_names)
          attr_reader(*attr_names)
          private(*attr_names)
          lambda do |*attr_values|
            attr_names.zip(attr_values) do |attr_name, attr_value|
              instance_variable_set(:"@#{attr_name}", attr_value)
            end
          end
        end

        # Defines the user's block as a method in a module this matcher
        # includes, then wraps it in a method of the matcher itself that can
        # call `super` to reach it.
        def define_user_override(method_name, user_def, &our_def)
          @user_method_defs.__send__(:define_method, method_name, &user_def)
          our_def ||= lambda { |*actual| super(*actual_arg_for(user_def)) }
          define_method(method_name, &our_def)
        end
      end

      module DefaultImplementations
        def description
          english_name = EnglishPhrasing.split_words(name)
          expected_list = EnglishPhrasing.list(expected_as_array)
          "#{english_name}#{expected_list}#{chained_method_clause_sentences}"
        end

        def failure_message
          "expected #{description_of @actual} to #{description}"
        end

        def failure_message_when_negated
          "expected #{description_of @actual} not to #{description}"
        end

        def diffable?
          false
        end

        def supports_block_expectations?
          false
        end

        def supports_value_expectations?
          true
        end

        def expects_call_stack_jump?
          false
        end

        private

        def chained_method_clause_sentences
          return "" unless Rex::Expectations.configuration.include_chain_clauses_in_custom_matcher_descriptions

          @chained_method_clauses.map do |(method_name, method_args)|
            " #{EnglishPhrasing.split_words(method_name)}#{EnglishPhrasing.list(method_args)}"
          end.join
        end
      end

      class Matcher
        include DefaultImplementations
        include Rex::Matchers
        include Rex::Matchers::Composable
        extend Macros

        attr_reader :expected_as_array, :rescued_exception, :name, :block_arg
        alias matcher_name name

        def initialize(name, declarations, matcher_execution_context, *expected, &block_arg)
          @name = name
          @actual = nil
          @expected_as_array = expected
          @matcher_execution_context = matcher_execution_context
          @chained_method_clauses = []
          @block_arg = block_arg

          klass = class << self
            self
          end
          klass.instance_variable_set(:@user_method_defs, Module.new)
          klass.__send__(:include, klass.instance_variable_get(:@user_method_defs))
          klass.class_exec(*expected, &declarations)
        end

        def actual
          @actual
        end

        def expected
          if expected_as_array.size == 1
            expected_as_array[0]
          else
            expected_as_array
          end
        end

        def inspect
          "#<#{self.class.name} #{name}>"
        end

        def respond_to_missing?(method, include_private = false)
          super || @matcher_execution_context.respond_to?(method, include_private)
        end

        private

        def actual_arg_for(block)
          block.arity.zero? ? [] : [actual]
        end

        def method_missing(method, *args, &block)
          if @matcher_execution_context.respond_to?(method)
            @matcher_execution_context.__send__(method, *args, &block)
          else
            super(method, *args, &block)
          end
        end
        ruby2_keywords(:method_missing) if respond_to?(:ruby2_keywords, true)
      end
    end

    extend DSL
  end
end
