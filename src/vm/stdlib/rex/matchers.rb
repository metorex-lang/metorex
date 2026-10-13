module Rex
  module Matchers
    class << self
      attr_accessor :last_matcher, :last_expectation_verb, :last_should_verb

      def configuration
        Expectations.configuration
      end

      # `it { is_expected.to eq 5 }` is described as "is expected to eq 5".
      def generated_description
        return nil unless last_matcher && last_expectation_verb

        description = last_matcher.respond_to?(:description) ? last_matcher.description : nil
        return nil if description.nil?

        verb = last_should_verb ? last_expectation_verb.sub("is expected", "should").sub("should to", "should").sub("should not to", "should not") : last_expectation_verb
        "#{verb} #{description}"
      end

      def clear_generated_description
        self.last_matcher = nil
        self.last_expectation_verb = nil
        self.last_should_verb = nil
      end

    end

    # The matcher-defining methods, which Rex::Matchers and every example
    # group answer.
    module DSL
      def alias_matcher(new_name, old_name, options = {}, &description_override)
        description_override ||= lambda do |old_description|
          old_description.gsub(EnglishPhrasing.split_words(old_name), EnglishPhrasing.split_words(new_name))
        end
        klass = options.fetch(:klass) { AliasedMatcher }
        define_method(new_name) do |*arguments, &block|
          matcher = __send__(old_name, *arguments, &block)
          matcher.matcher_name = new_name if matcher.respond_to?(:matcher_name=)
          klass.new(matcher, description_override)
        end
        ruby2_keywords(new_name) if respond_to?(:ruby2_keywords, true)
      end

      def define_negated_matcher(negated_name, base_name, &description_override)
        alias_matcher(negated_name, base_name, klass: AliasedNegatedMatcher, &description_override)
      end
    end
    extend DSL

    # Formats an object the way a failure message shows it.
    module ObjectFormatter
      MAX_LENGTH = 200
      ELLIPSIS = "..."

      def self.format(object)
        text = prepare_for_inspection(object)
        return text if text.length <= max_length

        keep = (max_length - ELLIPSIS.length) / 2
        text[0, keep] + ELLIPSIS + text[-keep..]
      end

      def self.max_length
        length = Rex::Expectations.configuration.max_formatted_output_length
        length.nil? && Rex::Expectations.configuration.max_formatted_output_length_set? ? Float::INFINITY : (length || MAX_LENGTH)
      end

      def self.prepare_for_inspection(object)
        prepare(object).inspect
      end

      # A copy of `object` that inspects the way a failure message shows it:
      # Hash keys sorted, matchers as their descriptions, and Times written
      # to the nanosecond.
      def self.prepare(object)
        case object
        when Hash
          pairs = object.to_a
          # Keys are sorted only when every one is a String or a Symbol.
          pairs = pairs.sort_by { |key, _| key.to_s } if object.keys.all? { |key| key.is_a?(String) || key.is_a?(Symbol) }
          pairs.to_h { |key, value| [prepare(key), prepare(value)] }
        when Array
          object.map { |item| prepare(item) }
        when Time
          InspectableItem.new(object.strftime("%Y-%m-%d %H:%M:%S.%N %z"))
        else
          if Matchers.matcher?(object) && object.respond_to?(:description)
            InspectableItem.new(object.description)
          else
            object
          end
        end
      end

      # Text that inspects as itself.
      InspectableItem = Struct.new(:text) do
        def inspect
          text
        end

        def pretty_print(printer)
          printer.text(text)
        end
      end
    end

    ::Rex::Support::ObjectFormatter = ObjectFormatter

    module EnglishPhrasing
      def self.split_words(symbol)
        symbol.to_s.tr("_", " ")
      end

      def self.list(items)
        return " #{ObjectFormatter.format(items)}" if !items.respond_to?(:to_ary) || items.is_a?(Struct)

        items = items.to_ary.map { |item| ObjectFormatter.format(item) }
        case items.length
        when 0 then ""
        when 1 then " #{items[0]}"
        when 2 then " #{items[0]} and #{items[1]}"
        else " #{items[0...-1].join(', ')}, and #{items[-1]}"
        end
      end
    end

    module FuzzyMatcher
      def self.values_match?(expected, actual)
        if actual.is_a?(Hash)
          return hashes_match?(expected, actual) if expected.is_a?(Hash)
        elsif expected.is_a?(Array) && actual.is_a?(Enumerable) && !actual.is_a?(Struct)
          return arrays_match?(expected, actual.to_a)
        end
        return true if expected == actual

        begin
          expected === actual
        rescue ArgumentError
          false
        end
      end

      def self.arrays_match?(expected, actual)
        return false if expected.size != actual.size

        expected.zip(actual).all? { |expected_item, actual_item| values_match?(expected_item, actual_item) }
      end

      def self.hashes_match?(expected, actual)
        return false if expected.size != actual.size

        expected.all? do |key, value|
          actual_value = actual.fetch(key) { return false }
          values_match?(value, actual_value)
        end
      end
    end

    def self.is_a_matcher?(object)
      return true if ::Rex::Matchers::BuiltIn::BaseMatcher === object

      begin
        return false if object.respond_to?(:i_respond_to_everything_so_im_not_really_a_matcher)
      rescue NoMethodError
        # An object such as a BasicObject answers no reflection methods.
        return false
      end
      return false unless object.respond_to?(:matches?)

      object.respond_to?(:failure_message) ||
        object.respond_to?(:failure_message_for_should)
    end

    ::Rex::Support.register_matcher_definition do |object|
      is_a_matcher?(object)
    end

    def self.is_a_describable_matcher?(object)
      is_a_matcher?(object) && object.respond_to?(:description)
    end

    class << self
      alias matcher? is_a_matcher?
    end

    # What lets a matcher stand where a value goes: `===`, `and`, `or`.
    module Composable
      def and(matcher)
        BuiltIn::Compound::And.new(self, matcher)
      end
      alias & and

      def or(matcher)
        BuiltIn::Compound::Or.new(self, matcher)
      end
      alias | or

      def ===(value)
        matches?(value)
      end

      def description_of(object)
        ObjectFormatter.format(object)
      end

      def values_match?(expected, actual)
        expected = with_matchers_cloned(expected)
        FuzzyMatcher.values_match?(expected, actual)
      end

      def with_matchers_cloned(object)
        if Matchers.matcher?(object)
          object.clone
        elsif Hash === object
          object.each_with_object({}) { |(key, value), hash| hash[key] = with_matchers_cloned(value) }
        elsif Array === object
          object.map { |item| with_matchers_cloned(item) }
        else
          object
        end
      end

      def surface_descriptions_in(item)
        if Matchers.matcher?(item) && item.respond_to?(:description)
          DescribableItem.new(item)
        elsif Hash === item
          surface_descriptions_in(item.to_a).to_h
        elsif Struct === item || unreadable_io?(item)
          ObjectFormatter.format(item)
        elsif should_enumerate?(item)
          item.map { |element| surface_descriptions_in(element) }
        else
          item
        end
      end

      def should_enumerate?(item)
        Array === item && item.none? { |element| element.equal?(item) }
      end

      # STDOUT answers `each` and raises IOError when it is called.
      def unreadable_io?(object)
        return false unless IO === object

        object.each {}
        false
      rescue IOError
        true
      end

      def self.surface_descriptions_in(item)
        Object.new.extend(self).surface_descriptions_in(item)
      end

      DescribableItem = Struct.new(:item) do
        def inspect
          "(#{item.description})"
        end

        def pretty_print(printer)
          printer.text "(#{item.description})"
        end
      end
    end

    module BuiltIn
      class BaseMatcher
        include Composable

        UNDEFINED = Object.new.freeze

        attr_reader :actual, :expected, :rescued_exception
        attr_writer :matcher_name

        def initialize(expected = UNDEFINED)
          @expected = expected unless UNDEFINED.equal?(expected)
        end

        def matches?(actual)
          @actual = actual
          match(expected, actual)
        end

        def match_unless_raises(*exceptions)
          exceptions.unshift Exception if exceptions.empty?
          begin
            yield
            true
          rescue *exceptions => @rescued_exception
            false
          end
        end

        def description
          desc = EnglishPhrasing.split_words(self.class.matcher_name)
          desc += EnglishPhrasing.list(@expected) if defined?(@expected)
          desc
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

        def expected_formatted
          Rex::Support::ObjectFormatter.format(@expected)
        end

        def actual_formatted
          Rex::Support::ObjectFormatter.format(@actual)
        end

        def self.matcher_name
          @matcher_name ||= underscore(name.split("::").last)
        end

        def matcher_name
          if defined?(@matcher_name)
            @matcher_name
          else
            self.class.matcher_name
          end
        end

        def self.underscore(camel_cased_word)
          word = camel_cased_word.to_s.dup
          word.gsub!(/([A-Z]+)([A-Z][a-z])/, '\1_\2')
          word.gsub!(/([a-z\d])([A-Z])/, '\1_\2')
          word.tr!("-", "_")
          word.downcase!
          word
        end
        private_class_method :underscore

        module HashFormatting
          def improve_hash_formatting(inspect_string)
            inspect_string.gsub(/(\S)=>(\S)/, '\1 => \2')
          end
          module_function :improve_hash_formatting
        end

        include HashFormatting

        module StringEncodingFormatting
          def string_encoding_differs?
            actual.is_a?(String) && expected.is_a?(String) && actual.encoding != expected.encoding
          end
          module_function :string_encoding_differs?

          def format_encoding(value)
            "#<Encoding:#{value.encoding.name}>"
          end
          module_function :format_encoding
        end

        include StringEncodingFormatting

        # The failure messages a matcher has when it does not write its own.
        module DefaultFailureMessages
          def failure_message
            +"expected #{description_of @actual} to #{description}"
          end

          def failure_message_when_negated
            +"expected #{description_of @actual} not to #{description}"
          end

          def self.has_default_failure_messages?(matcher)
            matcher.method(:failure_message).owner == self &&
              matcher.method(:failure_message_when_negated).owner == self
          rescue NameError
            false
          end
        end

        include DefaultFailureMessages
      end

      class Eq < BaseMatcher
        def failure_message
          "\nexpected: #{expected_formatted}\n     got: #{actual_formatted}\n\n(compared using ==)\n"
        end

        def failure_message_when_negated
          "\nexpected: value != #{expected_formatted}\n     got: #{actual_formatted}\n\n(compared using ==)\n"
        end

        def description
          "eq #{expected_formatted}"
        end

        def diffable?
          true
        end

        private

        def match(expected, actual)
          actual == expected
        end
      end

      class Eql < BaseMatcher
        def failure_message
          "\nexpected: #{expected_formatted}\n     got: #{actual_formatted}\n\n(compared using eql?)\n"
        end

        def failure_message_when_negated
          "\nexpected: value != #{expected_formatted}\n     got: #{actual_formatted}\n\n(compared using eql?)\n"
        end

        def diffable?
          true
        end

        private

        def match(expected, actual)
          actual.eql?(expected)
        end
      end

      class Equal < BaseMatcher
        LITERAL_SINGLETONS = [true, false, nil].freeze

        def failure_message
          if LITERAL_SINGLETONS.include?(expected)
            "\nexpected #{expected_formatted}\n     got #{actual_inspected}\n"
          else
            "\nexpected #{inspect_object(expected)}\n" \
              "     got #{inspect_object(actual)}\n" \
              "\n" \
              "Compared using equal?, which compares object identity,\n" \
              "but expected and actual are not the same object. Use\n" \
              "`expect(actual).to eq(expected)` if you don't care about\n" \
              "object identity in this example.\n"
          end
        end

        def failure_message_when_negated
          "\nexpected not #{inspect_object(actual)}\n" \
            "         got #{inspect_object(expected)}\n" \
            "\n" \
            "Compared using equal?, which compares object identity.\n"
        end

        def diffable?
          !LITERAL_SINGLETONS.include?(expected)
        end

        private

        def match(expected, actual)
          actual.equal?(expected)
        end

        def actual_inspected
          LITERAL_SINGLETONS.include?(actual) ? actual_formatted : inspect_object(actual)
        end

        def inspect_object(object)
          "#<#{object.class}:#{object.object_id}> => #{ObjectFormatter.format(object)}"
        end
      end

      class Be < BaseMatcher
        include Matchers::Composable

        def initialize(*arguments)
          @args = arguments
        end

        def failure_message
          "expected #{actual_formatted} to evaluate to true"
        end

        def failure_message_when_negated
          "expected #{actual_formatted} to evaluate to false"
        end

        %i[== < <= >= > === =~].each do |operator|
          define_method(operator) do |operand|
            BeComparedTo.new(operand, operator)
          end
        end

        private

        def match(_, actual)
          !!actual
        end
      end

      class BeComparedTo < BaseMatcher
        OPERATORS = %i[< > <= >=].freeze

        def initialize(operand, operator)
          @expected = operand
          @operator = operator
        end

        def matches?(actual)
          @actual = actual
          perform_match(actual)
        rescue ArgumentError, NoMethodError
          false
        end

        def does_not_match?(actual)
          @actual = actual
          !perform_match(actual)
        rescue ArgumentError, NoMethodError
          false
        end

        def failure_message
          "expected: #{@operator} #{expected_formatted}\n     got: #{@operator.to_s.gsub(/./, ' ')} #{actual_formatted}"
        end

        def failure_message_when_negated
          message = "`expect(#{actual_formatted}).not_to be #{@operator} #{expected_formatted}`"
          if OPERATORS.include?(@operator)
            message + " not only FAILED, it is a bit confusing."
          else
            message
          end
        end

        def description
          "be #{@operator} #{expected_formatted}"
        end

        private

        def perform_match(actual)
          actual.__send__(@operator, @expected) ? true : false
        end
      end

      class BeTruthy < BaseMatcher
        def failure_message
          "expected: truthy value\n     got: #{actual_formatted}"
        end

        def failure_message_when_negated
          "expected: falsey value\n     got: #{actual_formatted}"
        end

        private

        def match(_, actual)
          !!actual
        end
      end

      class BeFalsey < BaseMatcher
        def failure_message
          "expected: falsey value\n     got: #{actual_formatted}"
        end

        def failure_message_when_negated
          "expected: truthy value\n     got: #{actual_formatted}"
        end

        private

        def match(_, actual)
          !actual
        end
      end

      class BeNil < BaseMatcher
        def failure_message
          "expected: nil\n     got: #{actual_formatted}"
        end

        def failure_message_when_negated
          "expected: not nil\n     got: nil"
        end

        private

        def match(_, actual)
          actual.nil?
        end
      end

      class BeAKindOf < BaseMatcher
        private

        def match(expected, actual)
          actual.kind_of?(expected)
        rescue NoMethodError
          raise ArgumentError, "The #{matcher_name} matcher requires that the actual object responds to #kind_of? method but a `NoMethodError` was encountered instead."
        end
      end

      class BeAnInstanceOf < BaseMatcher
        def description
          "be an instance of #{expected}"
        end

        private

        def match(expected, actual)
          actual.instance_of?(expected)
        end
      end

      class BeBetween < BaseMatcher
        def initialize(min, max)
          @min = min
          @max = max
          inclusive
        end

        def inclusive
          @less_than_operator = :<=
          @greater_than_operator = :>=
          @mode = :inclusive
          self
        end

        def exclusive
          @less_than_operator = :<
          @greater_than_operator = :>
          @mode = :exclusive
          self
        end

        def matches?(actual)
          @actual = actual
          comparable? && compare
        rescue ArgumentError
          false
        end

        def failure_message
          "#{super}#{not_comparable_clause}"
        end

        def description
          "be between #{description_of @min} and #{description_of @max} (#{@mode})"
        end

        private

        def comparable?
          @actual.respond_to?(@less_than_operator) && @actual.respond_to?(@greater_than_operator)
        end

        def not_comparable_clause
          ", but it does not respond to `#{@less_than_operator}` and `#{@greater_than_operator}`" unless comparable?
        end

        def compare
          @actual.__send__(@greater_than_operator, @min) && @actual.__send__(@less_than_operator, @max)
        end
      end

      class BeWithin < BaseMatcher
        def initialize(delta)
          @delta = delta
        end

        def of(expected)
          @expected = expected
          @tolerance = @delta
          @unit = ""
          self
        end

        def percent_of(expected)
          @expected = expected
          @tolerance = @expected.abs * @delta / 100.0
          @unit = "%"
          self
        end

        def matches?(actual)
          @actual = actual
          raise needs_expected unless defined?(@expected)

          numeric? && (@actual - @expected).abs <= @tolerance
        end

        def failure_message
          "expected #{actual_formatted} to #{description}#{not_numeric_clause}"
        end

        def failure_message_when_negated
          "expected #{actual_formatted} not to #{description}"
        end

        def description
          "be within #{@delta}#{@unit} of #{expected_formatted}"
        end

        private

        def numeric?
          @actual.respond_to?(:-)
        end

        def needs_expected
          ArgumentError.new("You must set an expected value using #of: be_within(#{@delta}).of(expected_value)")
        end

        def not_numeric_clause
          ", but it could not be treated as a numeric value" unless numeric?
        end
      end

      class BePredicate < BaseMatcher
        def initialize(method_name, *arguments, &block)
          @method_name = method_name
          @args = arguments
          @block = block
          @prefix, @expected = /\A(be_(?:an?_)?)(.*)/.match(method_name.to_s).captures
        end
        ruby2_keywords(:initialize) if respond_to?(:ruby2_keywords, true)

        def matches?(actual, &block)
          @actual = actual
          @block ||= block
          predicate_accessible? && predicate_matches?
        end

        def does_not_match?(actual, &block)
          @actual = actual
          @block ||= block
          predicate_accessible? && !predicate_matches?
        end

        def failure_message
          failure_message_expecting(true)
        end

        def failure_message_when_negated
          failure_message_expecting(false)
        end

        def description
          "#{EnglishPhrasing.split_words(@method_name)}#{EnglishPhrasing.list(@args) unless @args.empty?}"
        end

        private

        def predicate
          :"#{@expected}?"
        end

        def present_tense_predicate
          :"#{@expected}s?"
        end

        def predicate_accessible?
          @actual.respond_to?(predicate) || @actual.respond_to?(present_tense_predicate)
        end

        def predicate_method_name
          @actual.respond_to?(predicate) ? predicate : present_tense_predicate
        end

        def predicate_result
          @predicate_result = @actual.__send__(predicate_method_name, *@args, &@block)
        end

        def predicate_matches?
          result = predicate_result
          Rex::Expectations.configuration.strict_predicate_matchers ? result == true : !!result
        end

        def failure_message_expecting(value)
          return "expected #{actual_formatted} to respond to `#{predicate}`" unless predicate_accessible?

          validity_message = predicate_accessible? ? "" : ""
          "expected `#{actual_formatted}.#{predicate}#{args_to_s}` to #{expectation_of value}, got #{description_of @predicate_result}#{validity_message}"
        end

        def expectation_of(value)
          if Rex::Expectations.configuration.strict_predicate_matchers
            "return #{value}"
          elsif value
            "be truthy"
          else
            "be falsey"
          end
        end

        def args_to_s
          @args.empty? ? "" : "(#{@args.map(&:inspect).join(', ')})"
        end
      end

      class Has < BePredicate
        def initialize(method_name, *arguments, &block)
          @method_name = method_name
          @args = arguments
          @block = block
          @expected = method_name.to_s.sub(/\Ahave_/, "has_")
        end
        ruby2_keywords(:initialize) if respond_to?(:ruby2_keywords, true)

        private

        def predicate
          :"#{@expected}?"
        end

        def predicate_accessible?
          @actual.respond_to?(predicate)
        end

        def predicate_method_name
          predicate
        end

        def failure_message_expecting(value)
          return "expected #{actual_formatted} to respond to `#{predicate}`" unless predicate_accessible?

          "expected `#{actual_formatted}.#{predicate}#{args_to_s}` to #{expectation_of value}, got #{description_of @predicate_result}"
        end
      end

      class RespondTo < BaseMatcher
        def initialize(*names)
          @names = names
          @expected_arity = nil
          @expected_keywords = []
          @unlimited_arguments = nil
          @arbitrary_keywords = nil
        end

        def with(count)
          @expected_arity = count
          self
        end

        def argument
          self
        end
        alias arguments argument
        alias and with

        def with_keywords(*keywords)
          @expected_keywords = keywords
          self
        end
        alias and_keywords with_keywords

        def with_unlimited_arguments
          @unlimited_arguments = true
          self
        end
        alias and_unlimited_arguments with_unlimited_arguments

        def with_any_keywords
          @arbitrary_keywords = true
          self
        end
        alias and_any_keywords with_any_keywords

        def matches?(actual)
          find_failing_method_names(actual, :reject).empty?
        end

        def does_not_match?(actual)
          find_failing_method_names(actual, :select).empty?
        end

        def failure_message
          "expected #{actual_formatted} to respond to #{@failing_method_names.map(&:inspect).join(', ')}#{with_arity}"
        end

        def failure_message_when_negated
          failure_message.sub("to respond to", "not to respond to")
        end

        def description
          "respond to #{pp_names}#{with_arity}"
        end

        private

        def find_failing_method_names(actual, filter_method)
          @actual = actual
          @failing_method_names = @names.__send__(filter_method) do |name|
            @actual.respond_to?(name) && matches_arity?(actual, name)
          end
        end

        def matches_arity?(actual, name)
          return true unless @expected_arity || !@expected_keywords.empty? || @unlimited_arguments || @arbitrary_keywords

          parameters = Rex::Core::MethodSignature.new(actual.method(name))
          parameters.accepts?(@expected_arity, @expected_keywords, @unlimited_arguments, @arbitrary_keywords)
        rescue NameError
          true
        end

        def with_arity
          clauses = []
          clauses << " with #{@expected_arity} argument#{'s' unless @expected_arity == 1}" if @expected_arity
          clauses << " with unlimited arguments" if @unlimited_arguments
          clauses << " with keyword#{'s' unless @expected_keywords.size == 1} #{@expected_keywords.map(&:inspect).join(' and ')}" unless @expected_keywords.empty?
          clauses << " with any keywords" if @arbitrary_keywords
          clauses.join(" and").sub(/\A and/, "")
        end

        def pp_names
          @names.length == 1 ? "##{@names.first}" : @names.map(&:inspect).join(", ")
        end
      end

      # `once`, `twice`, `at_least(n).times` and the rest, for the matchers
      # that count what they find.
      module CountExpectation
        def once
          exactly(1)
        end

        def twice
          exactly(2)
        end

        def thrice
          exactly(3)
        end

        def exactly(number)
          set_expected_count(:==, number)
          self
        end

        def at_most(number)
          set_expected_count(:<=, number)
          self
        end

        def at_least(number)
          set_expected_count(:>=, number)
          self
        end

        def times
          self
        end

      protected
        attr_reader :count_expectation_type, :expected_count

      private

        def cover?(count, number)
          count.cover?(number)
        end

        def expected_count_matches?(actual_count)
          @actual_count = actual_count
          return @actual_count > 0 unless count_expectation_type
          return cover?(expected_count, actual_count) if count_expectation_type == :<=>

          @actual_count.__send__(count_expectation_type, expected_count)
        end

        def has_expected_count?
          !!count_expectation_type
        end

        def set_expected_count(relativity, n)
          raise_unsupported_count_expectation if unsupported_count_expectation?(relativity)

          count = count_constraint_to_number(n)

          if count_expectation_type == :<= && relativity == :>=
            raise_impossible_count_expectation(count) if count > expected_count
            @count_expectation_type = :<=>
            @expected_count = count..expected_count
          elsif count_expectation_type == :>= && relativity == :<=
            raise_impossible_count_expectation(count) if count < expected_count
            @count_expectation_type = :<=>
            @expected_count = expected_count..count
          else
            @count_expectation_type = relativity
            @expected_count = count
          end
        end

        def raise_impossible_count_expectation(count)
          text =
            case count_expectation_type
            when :<= then "at_least(#{count}).at_most(#{expected_count})"
            when :>= then "at_least(#{expected_count}).at_most(#{count})"
            end
          raise ArgumentError, "The constraint #{text} is not possible"
        end

        def raise_unsupported_count_expectation
          text =
            case count_expectation_type
            when :<= then "at_least"
            when :>= then "at_most"
            when :<=> then "at_least/at_most combination"
            else "count"
            end
          raise ArgumentError, "Multiple #{text} constraints are not supported"
        end

        def count_constraint_to_number(n)
          case n
          when Numeric then n
          when :once then 1
          when :twice then 2
          when :thrice then 3
          else
            raise ArgumentError, "Expected a number, :once, :twice or :thrice," \
              " but got #{n}"
          end
        end

        def unsupported_count_expectation?(relativity)
          return true if count_expectation_type == :==
          return true if count_expectation_type == :<=>
          (count_expectation_type == :<= && relativity == :<=) ||
            (count_expectation_type == :>= && relativity == :>=)
        end

        def count_expectation_description
          "#{human_readable_expectation_type}#{human_readable_count(expected_count)}"
        end

        def count_failure_reason(action)
          "#{count_expectation_description}" \
          " but #{action}#{human_readable_count(@actual_count)}"
        end

        def human_readable_expectation_type
          case count_expectation_type
          when :<= then ' at most'
          when :>= then ' at least'
          when :<=> then ' between'
          else ''
          end
        end

        def human_readable_count(count)
          case count
          when Range then " #{count.first} and #{count.last} times"
          when nil then ''
          when 1 then ' once'
          when 2 then ' twice'
          else " #{count} times"
          end
        end
      end

      class Include < BaseMatcher
        include CountExpectation
        attr_reader :expecteds

        def initialize(*expecteds)
          raise(ArgumentError, 'include() is not supported, please supply an argument') if expecteds.empty?
          @expecteds = expecteds
        end

        def matches?(actual)
          check_actual?(actual) &&
            if check_expected_count?
              expected_count_matches?(count_inclusions)
            else
              perform_match { |v| v }
            end
        end

        def does_not_match?(actual)
          check_actual?(actual) &&
            if check_expected_count?
              !expected_count_matches?(count_inclusions)
            else
              perform_match { |v| !v }
            end
        end

        def description
          improve_hash_formatting("include#{readable_list_of(expecteds)}#{count_expectation_description}")
        end

        def failure_message
          format_failure_message("to") { super }
        end

        def failure_message_when_negated
          format_failure_message("not to") { super }
        end

        def diffable?
          !diff_would_wrongly_highlight_matched_item?
        end

        def expected
          if expecteds.one? && Hash === expecteds.first
            expecteds.first
          else
            expecteds
          end
        end

        private

        def check_actual?(actual)
          actual = actual.to_hash if convert_to_hash?(actual)
          @actual = actual
          @actual.respond_to?(:include?)
        end

        def check_expected_count?
          case
          when !has_expected_count?
            return false
          when expecteds.size != 1
            raise NotImplementedError, 'Count constraint supported only when testing for a single value being included'
          when actual.is_a?(Hash)
            raise NotImplementedError, 'Count constraint on hash keys not implemented'
          end
          true
        end

        def format_failure_message(preposition)
          msg = if actual.respond_to?(:include?)
                  "expected #{description_of @actual} #{preposition}" \
                  " include#{readable_list_of @divergent_items}" \
                  "#{count_failure_reason('it is included') if has_expected_count?}"
                else
                  "#{yield}, but it does not respond to `include?`"
                end
          improve_hash_formatting(msg)
        end

        def readable_list_of(items)
          described_items = surface_descriptions_in(items)
          if described_items.all? { |item| item.is_a?(Hash) }
            " #{described_items.inject(:merge).inspect}"
          else
            EnglishPhrasing.list(described_items)
          end
        end

        def perform_match(&block)
          @divergent_items = excluded_from_actual(&block)
          @divergent_items.empty?
        end

        def excluded_from_actual
          return [] unless @actual.respond_to?(:include?)

          expecteds.inject([]) do |memo, expected_item|
            if comparing_hash_to_a_subset?(expected_item)
              expected_item.each do |(key, value)|
                memo << { key => value } unless yield actual_hash_includes?(key, value)
              end
            elsif comparing_hash_keys?(expected_item)
              memo << expected_item unless yield actual_hash_has_key?(expected_item)
            else
              memo << expected_item unless yield actual_collection_includes?(expected_item)
            end
            memo
          end
        end

        def comparing_hash_to_a_subset?(expected_item)
          actual.is_a?(Hash) && expected_item.is_a?(Hash)
        end

        def actual_hash_includes?(expected_key, expected_value)
          actual_value =
            actual.fetch(expected_key) do
              actual.find(Proc.new { return false }) { |actual_key, _| values_match?(expected_key, actual_key) }[1]
            end
          values_match?(expected_value, actual_value)
        end

        def comparing_hash_keys?(expected_item)
          actual.is_a?(Hash) && !expected_item.is_a?(Hash)
        end

        def actual_hash_has_key?(expected_key)
          has_exact_key =
            begin
              actual.key?(expected_key)
            rescue
              false
            end

          has_exact_key || actual.keys.any? { |key| values_match?(expected_key, key) }
        end

        def actual_collection_includes?(expected_item)
          return actual.scan(expected_item).size > 0 if Regexp === expected_item && String === actual
          return true if actual.include?(expected_item)

          return false unless actual.respond_to?(:any?)

          actual.any? { |value| values_match?(expected_item, value) }
        end

        def count_enumerable(expected_item)
          actual.count { |value| values_match?(expected_item, value) }
        end

        def count_inclusions
          @divergent_items = expected
          case actual
          when String
            actual.scan(expected.first).length
          when Enumerable
            count_enumerable(Hash === expected ? expected : expected.first)
          else
            raise NotImplementedError, 'Count constraints are implemented for Enumerable and String values only'
          end
        end

        def diff_would_wrongly_highlight_matched_item?
          return false unless actual.is_a?(String) && expected.is_a?(Array)
          return false if Regexp === expecteds.first

          lines = actual.split("\n")
          expected.any? do |str|
            actual.include?(str) && lines.none? { |line| line == str }
          end
        end

        def convert_to_hash?(obj)
          !obj.respond_to?(:include?) && obj.respond_to?(:to_hash)
        end
      end

      class ContainExactly < BaseMatcher
        def failure_message
          if Array === actual
            generate_failure_message
          else
            "expected a collection that can be converted to an array with " \
              "`#to_ary` or `#to_a`, but got #{actual_formatted}"
          end
        end

        def failure_message_when_negated
          list = EnglishPhrasing.list(surface_descriptions_in(expected))
          "expected #{actual_formatted} not to contain exactly#{list}"
        end

        def description
          list = EnglishPhrasing.list(surface_descriptions_in(expected))
          "contain exactly#{list}"
        end

        def matches?(actual)
          @pairings_maximizer = nil
          @best_solution = nil
          @extra_items = nil
          @missing_items = nil
          super(actual)
        end

        private

        def generate_failure_message
          message = expected_collection_line
          message += actual_collection_line
          message += missing_elements_line unless missing_items.empty?
          message += extra_elements_line unless extra_items.empty?
          message
        end

        def expected_collection_line
          message_line("expected collection contained", expected, true)
        end

        def actual_collection_line
          message_line("actual collection contained", actual)
        end

        def missing_elements_line
          message_line("the missing elements were", missing_items, true)
        end

        def extra_elements_line
          message_line("the extra elements were", extra_items)
        end

        def describe_collection(collection, surface_descriptions = false)
          if surface_descriptions
            "#{description_of(safe_sort(surface_descriptions_in(collection)))}\n"
          else
            "#{description_of(safe_sort(collection))}\n"
          end
        end

        def message_line(prefix, collection, surface_descriptions = false)
          format("%-32s%s", "#{prefix}:", describe_collection(collection, surface_descriptions))
        end

        def match(_expected, _actual)
          return false unless convert_actual_to_an_array

          match_when_sorted? || (extra_items.empty? && missing_items.empty?)
        end

        def match_when_sorted?
          values_match?(safe_sort(expected), safe_sort(actual))
        end

        def convert_actual_to_an_array
          if actual.respond_to?(:to_ary)
            @actual = actual.to_ary
          elsif actual.respond_to?(:to_a) && !to_a_disallowed?(actual)
            @actual = actual.to_a
          else
            false
          end
        end

        def safe_sort(array)
          array.sort
        rescue Rex::Support::AllExceptionsExceptOnesWeMustNotRescue
          array
        end

        def to_a_disallowed?(object)
          NilClass === object
        end

        def missing_items
          @missing_items ||= best_solution.unmatched_expected_indexes.map do |index|
            expected[index]
          end
        end

        def extra_items
          @extra_items ||= best_solution.unmatched_actual_indexes.map do |index|
            actual[index]
          end
        end

        def best_solution
          @best_solution ||= pairings_maximizer.find_best_solution
        end

        def pairings_maximizer
          @pairings_maximizer ||= begin
            expected_matches = Array.new(expected.size) { |index| [index, []] }.to_h
            actual_matches = Array.new(actual.size) { |index| [index, []] }.to_h

            expected.each_with_index do |expected_item, expected_index|
              actual.each_with_index do |actual_item, actual_index|
                next unless values_match?(expected_item, actual_item)

                expected_matches[expected_index] << actual_index
                actual_matches[actual_index] << expected_index
              end
            end

            PairingsMaximizer.new(expected_matches, actual_matches)
          end
        end

        # Pairs each expected item with an actual item it matches so that
        # as few items as possible are left unpaired, trying each pairing
        # of an item that matches more than one.
        class PairingsMaximizer
          Solution = Struct.new(:unmatched_expected_indexes, :unmatched_actual_indexes,
                                :indeterminate_expected_indexes, :indeterminate_actual_indexes) do
            def worse_than?(other)
              unmatched_item_count > other.unmatched_item_count
            end

            def candidate?
              indeterminate_expected_indexes.empty? &&
                indeterminate_actual_indexes.empty?
            end

            def ideal?
              candidate? && (
                unmatched_expected_indexes.empty? ||
                unmatched_actual_indexes.empty?
              )
            end

            def unmatched_item_count
              unmatched_expected_indexes.count + unmatched_actual_indexes.count
            end

            def +(other)
              self.class.new(
                unmatched_expected_indexes + other.unmatched_expected_indexes,
                unmatched_actual_indexes + other.unmatched_actual_indexes,
                [], []
              )
            end
          end

          attr_reader :expected_to_actual_matched_indexes, :actual_to_expected_matched_indexes, :solution

          def initialize(expected_to_actual_matched_indexes, actual_to_expected_matched_indexes)
            @expected_to_actual_matched_indexes = expected_to_actual_matched_indexes
            @actual_to_expected_matched_indexes = actual_to_expected_matched_indexes

            unmatched_expected_indexes, indeterminate_expected_indexes =
              categorize_indexes(expected_to_actual_matched_indexes, actual_to_expected_matched_indexes)

            unmatched_actual_indexes, indeterminate_actual_indexes =
              categorize_indexes(actual_to_expected_matched_indexes, expected_to_actual_matched_indexes)

            @solution = Solution.new(unmatched_expected_indexes, unmatched_actual_indexes,
                                     indeterminate_expected_indexes, indeterminate_actual_indexes)
          end

          def find_best_solution
            return solution if solution.candidate?

            best_solution_so_far = NullSolution

            expected_index = solution.indeterminate_expected_indexes.first
            actuals = expected_to_actual_matched_indexes[expected_index]

            actuals.each do |actual_index|
              solution = best_solution_for_pairing(expected_index, actual_index)
              return solution if solution.ideal?

              best_solution_so_far = solution if best_solution_so_far.worse_than?(solution)
            end

            best_solution_so_far
          end

          NullSolution = Class.new do
            def self.worse_than?(_other)
              true
            end
          end
          private_constant :NullSolution

          private

          def categorize_indexes(indexes_to_categorize, other_indexes)
            unmatched = []
            indeterminate = []

            indexes_to_categorize.each_pair do |index, matches|
              if matches.empty?
                unmatched << index
              elsif !reciprocal_single_match?(matches, index, other_indexes)
                indeterminate << index
              end
            end

            return unmatched, indeterminate
          end

          def reciprocal_single_match?(matches, index, other_list)
            return false unless matches.one?

            other_list[matches.first] == [index]
          end

          def best_solution_for_pairing(expected_index, actual_index)
            modified_expecteds = apply_pairing_to(
              solution.indeterminate_expected_indexes,
              expected_to_actual_matched_indexes, actual_index
            )
            modified_expecteds.delete(expected_index)

            modified_actuals = apply_pairing_to(
              solution.indeterminate_actual_indexes,
              actual_to_expected_matched_indexes, expected_index
            )
            modified_actuals.delete(actual_index)

            solution + self.class.new(modified_expecteds, modified_actuals).find_best_solution
          end

          def apply_pairing_to(indeterminates, original_matches, other_list_index)
            indeterminates.each_with_object({}) do |index, accumulated|
              accumulated[index] = original_matches[index] - [other_list_index]
            end
          end
        end
      end

      class StartOrEndWith < BaseMatcher
        def initialize(*expected)
          @actual_does_not_have_ordered_elements = false
          @expected = expected.length == 1 ? expected.first : expected
        end

        def failure_message
          "expected #{description_of @actual} to #{description}#{reason}"
        end

        def description
          "#{EnglishPhrasing.split_words(self.class.matcher_name)}#{EnglishPhrasing.list(expected.is_a?(Array) ? expected : [expected])}"
        end

        private

        def reason
          if @actual_does_not_have_ordered_elements
            ", but it does not have ordered elements"
          elsif !@actual.respond_to?(:[])
            ", but it cannot be indexed using #[]"
          else
            ""
          end
        end

        def match(_expected, actual)
          return false unless actual.respond_to?(:length) && actual.respond_to?(:[])

          begin
            return true if subsets_comparable? && subset_matches?
            element_matches?
          rescue ArgumentError
            @actual_does_not_have_ordered_elements = true
            false
          end
        end

        def subsets_comparable?
          expected.respond_to?(:length)
        end
      end

      class StartWith < StartOrEndWith
        private

        def subset_matches?
          values_match?(expected, actual[0, expected.length])
        end

        def element_matches?
          values_match?(expected, actual[0])
        end
      end

      class EndWith < StartOrEndWith
        private

        def subset_matches?
          values_match?(expected, actual[-expected.length..-1])
        end

        def element_matches?
          values_match?(expected, actual[-1])
        end
      end

      class All < BaseMatcher
        attr_reader :matcher, :failed_objects

        def initialize(matcher)
          @matcher = matcher
          @failed_objects = {}
        end

        def does_not_match?(_actual)
          raise NotImplementedError, "`expect().not_to all( matcher )` is not supported."
        end

        def failure_message
          unless iterable?
            return "#{improve_hash_formatting(super)}, but was not iterable"
          end

          all_messages = [improve_hash_formatting(super)]
          failed_objects.each do |index, matcher_failure_message|
            all_messages << failure_message_for_item(index, matcher_failure_message)
          end
          all_messages.join("\n\n")
        end

        def description
          improve_hash_formatting "all #{description_of matcher}"
        end

        private

        def match(_expected, _actual)
          return false unless iterable?

          index_failed_objects
          failed_objects.empty?
        end

        def index_failed_objects
          actual.each_with_index do |actual_item, index|
            cloned_matcher = matcher.clone
            matches = cloned_matcher.matches?(actual_item)
            failed_objects[index] = cloned_matcher.failure_message unless matches
          end
        end

        def failure_message_for_item(index, failure_message)
          failure_message = indent_multiline_message(add_new_line_if_needed(failure_message))
          indent_multiline_message("object at index #{index} failed to match:#{failure_message}")
        end

        def add_new_line_if_needed(message)
          message.start_with?("\n") ? message : "\n#{message}"
        end

        def indent_multiline_message(message)
          message = message.sub(/\n+\z/, "")
          message.lines.map do |line|
            line =~ /\S/ ? "   " + line : line
          end.join
        end

        def iterable?
          @actual.respond_to?(:each_with_index)
        end

        def improve_hash_formatting(text)
          text
        end
      end

      class Match < BaseMatcher
        def description
          if @expected.is_a?(Regexp)
            "match #{expected_formatted}"
          else
            "match #{surface_descriptions_in(expected).inspect}"
          end
        end

        def diffable?
          true
        end

        def with_captures(*captures)
          @expected_captures = captures
          self
        end

        private

        def match(expected, actual)
          return match_captures(expected, actual) if @expected_captures
          return true if values_match?(expected, actual)
          return false if Array === expected || Hash === expected

          actual.match(expected) if actual.respond_to?(:match)
        end

        def match_captures(expected, actual)
          match = actual.match(expected)
          if match
            match = match.names.empty? ? match.captures : match.named_captures.transform_keys(&:to_sym)
            values_match?(@expected_captures, Array === @expected_captures.first ? match.to_a : match)
          else
            false
          end
        end
      end

      class RaiseError
        include Composable

        # What `raise_error` is handed when it is given nothing, told apart
        # from `nil`, which warns differently.
        UndefinedValue = Object.new.freeze

        def initialize(expected_error_or_message, expected_message, &block)
          @block = block
          @actual_error = nil
          @warn_about_bare_error = UndefinedValue === expected_error_or_message
          @warn_about_nil_error = expected_error_or_message.nil?

          case expected_error_or_message
          when nil, UndefinedValue
            @expected_error = Exception
            @expected_message = expected_message
          when String, Regexp
            @expected_error = Exception
            @expected_message = expected_error_or_message
          else
            @expected_error = expected_error_or_message
            @expected_message = expected_message
          end
        end

        def with_message(expected_message)
          raise_message_already_set if @expected_message
          @warn_about_bare_error = false
          @expected_message = expected_message
          self
        end

        def matches?(given_proc, negative_expectation=false, &block)
          @given_proc = given_proc
          @block ||= block
          @raised_expected_error = false
          @with_expected_message = false
          @eval_block = false
          @eval_block_passed = false

          return false unless Proc === given_proc

          begin
            given_proc.call
          rescue Exception => @actual_error
            if values_match?(@expected_error, @actual_error) ||
               values_match?(@expected_error, actual_error_message)
              @raised_expected_error = true
              @with_expected_message = verify_message
            end
          end

          unless negative_expectation
            warn_about_bare_error! if warn_about_bare_error?
            warn_about_nil_error! if warn_about_nil_error?
            eval_block if ready_to_eval_block?
          end

          expectation_matched?
        end

        def does_not_match?(given_proc)
          warn_for_negative_false_positives!
          !matches?(given_proc, :negative_expectation) && Proc === given_proc
        end

        def supports_block_expectations?
          true
        end

        def supports_value_expectations?
          false
        end

        def expects_call_stack_jump?
          true
        end

        def failure_message
          @eval_block ? actual_error_message : "expected #{expected_error}#{given_error}"
        end

        def failure_message_when_negated
          "expected no #{expected_error}#{given_error}"
        end

        def description
          "raise #{expected_error}"
        end

        private

        def actual_error_message
          return nil unless @actual_error

          @actual_error.respond_to?(:original_message) ? @actual_error.original_message : @actual_error.message
        end

        def expectation_matched?
          error_and_message_match? && block_matches?
        end

        def error_and_message_match?
          @raised_expected_error && @with_expected_message
        end

        def block_matches?
          @eval_block ? @eval_block_passed : true
        end

        def ready_to_eval_block?
          @raised_expected_error && @with_expected_message && @block
        end

        def eval_block
          @eval_block = true
          begin
            @block[@actual_error]
            @eval_block_passed = true
          rescue Exception => err
            @actual_error = err
          end
        end

        def verify_message
          return true if @expected_message.nil?
          values_match?(@expected_message, actual_error_message.to_s)
        end

        def warn_for_negative_false_positives!
          expression = if expecting_specific_exception? && @expected_message
                         "`expect { }.not_to raise_error(SpecificErrorClass, message)`"
                       elsif expecting_specific_exception?
                         "`expect { }.not_to raise_error(SpecificErrorClass)`"
                       elsif @expected_message
                         "`expect { }.not_to raise_error(message)`"
                       elsif @warn_about_nil_error
                         "`expect { }.not_to raise_error(nil)`"
                       end

          return unless expression

          warn_about_negative_false_positive! expression
        end

        def handle_warning(message)
          Rex::Expectations.configuration.false_positives_handler.call(message)
        end

        def warn_about_bare_error?
          @warn_about_bare_error && @block.nil?
        end

        def warn_about_nil_error?
          @warn_about_nil_error
        end

        def warn_about_bare_error!
          handle_warning("Using the `raise_error` matcher without providing a specific " \
                         "error or message risks false positives, since `raise_error` " \
                         "will match when Ruby raises a `NoMethodError`, `NameError` or " \
                         "`ArgumentError`, potentially allowing the expectation to pass " \
                         "without even executing the method you are intending to call. " \
                         "#{warning}"\
                         "Instead consider providing a specific error class or message. " \
                         "This message can be suppressed by setting: " \
                         "`RSpec::Expectations.configuration.on_potential_false" \
                         "_positives = :nothing`")
        end

        def warn_about_nil_error!
          handle_warning("Using the `raise_error` matcher with a `nil` error is probably " \
                         "unintentional, it risks false positives, since `raise_error` " \
                         "will match when Ruby raises a `NoMethodError`, `NameError` or " \
                         "`ArgumentError`, potentially allowing the expectation to pass " \
                         "without even executing the method you are intending to call. " \
                         "#{warning}"\
                         "Instead consider providing a specific error class or message. " \
                         "This message can be suppressed by setting: " \
                         "`RSpec::Expectations.configuration.on_potential_false" \
                         "_positives = :nothing`")
        end

        def warn_about_negative_false_positive!(expression)
          handle_warning("Using #{expression} risks false positives, since literally " \
                         "any other error would cause the expectation to pass, " \
                         "including those raised by Ruby (e.g. `NoMethodError`, `NameError` " \
                         "and `ArgumentError`), meaning the code you are intending to test " \
                         "may not even get reached. Instead consider using " \
                         "`expect { }.not_to raise_error` or `expect { }.to raise_error" \
                         "(DifferentSpecificErrorClass)`. This message can be suppressed by " \
                         "setting: `RSpec::Expectations.configuration.on_potential_false" \
                         "_positives = :nothing`")
        end

        def expected_error
          case @expected_message
          when nil
            if Matchers.matcher?(@expected_error)
              "Exception with #{description_of(@expected_error)}"
            else
              description_of(@expected_error)
            end
          when Regexp
            "#{@expected_error} with message matching #{description_of(@expected_message)}"
          else
            "#{@expected_error} with #{description_of(@expected_message)}"
          end
        end

        def format_backtrace(backtrace)
          Rex.configuration.backtrace_formatter.format_backtrace(backtrace)
        end

        def given_error
          return " but was not given a block" unless Proc === @given_proc
          return " but nothing was raised" unless @actual_error

          backtrace = format_backtrace(@actual_error.backtrace)
          [
            ", got #{description_of(@actual_error)} with backtrace:",
            *backtrace
          ].join("\n  # ")
        end

        def expecting_specific_exception?
          @expected_error != Exception
        end

        def raise_message_already_set
          raise "`expect { }.to raise_error(message).with_message(message)` is not valid. " \
                'The matcher only allows the expected message to be specified once'
        end

        def warning
          warning = "Actual error raised was #{description_of(@actual_error)}. "
          warning if @actual_error
        end
      end

      class ThrowSymbol
        include Composable

        def initialize(expected_symbol = nil, expected_arg = nil)
          @expected_symbol = expected_symbol
          @expected_arg = expected_arg
          @caught_symbol = @caught_arg = nil
        end

        def matches?(given_proc)
          @block = given_proc
          return false unless Proc === given_proc

          begin
            if @expected_symbol.nil?
              given_proc.call
            else
              @caught_arg = catch(@expected_symbol) do
                given_proc.call
                return false
              end
              @caught_symbol = @expected_symbol
            end
          rescue UncaughtThrowError => error
            @caught_symbol = error.tag
          end

          if @expected_symbol.nil?
            !!@caught_symbol
          elsif @expected_arg.nil?
            @caught_symbol == @expected_symbol
          else
            (@caught_symbol == @expected_symbol) && values_match?(@expected_arg, @caught_arg)
          end
        end

        def does_not_match?(given_proc)
          !matches?(given_proc) && Proc === given_proc
        end

        def failure_message
          "expected #{expected} to be thrown, #{actual_result}"
        end

        def failure_message_when_negated
          "expected #{expected('no ')} to be thrown, #{actual_result}"
        end

        def description
          "throw #{expected}"
        end

        def supports_block_expectations?
          true
        end

        def supports_value_expectations?
          false
        end

        def expects_call_stack_jump?
          true
        end

        private

        def actual_result
          return "but was not a block" unless Proc === @block

          "got #{caught}"
        end

        def caught
          if @caught_symbol
            "#{@caught_symbol.inspect}#{@caught_arg.nil? ? '' : " with #{description_of @caught_arg}"}"
          else
            "nothing"
          end
        end

        def expected(symbol_desc = "a ")
          throw_description(@expected_symbol || "#{symbol_desc}Symbol", @expected_arg)
        end

        def throw_description(symbol, arg)
          symbol_description = symbol.is_a?(String) ? symbol : symbol.inspect
          arg_description = arg.nil? ? "" : " with #{description_of arg}"
          "#{symbol_description}#{arg_description}"
        end
      end

      class HaveAttributes < BaseMatcher
        attr_reader :respond_to_failed

        def initialize(expected)
          @expected = expected
          @values = {}
          @respond_to_failed = false
          @negated = false
        end

        def actual
          @values
        end

        def matches?(actual)
          @actual = actual
          @negated = false
          respond_to_attributes? && perform_match(:all?)
        end

        def does_not_match?(actual)
          @actual = actual
          @negated = true
          respond_to_attributes? && perform_match(:none?)
        end

        def description
          "have attributes #{ObjectFormatter.format(expected)}"
        end

        def diffable?
          !@respond_to_failed && !@negated
        end

        def failure_message
          respond_to_failure_message_or do
            "expected #{actual_formatted} to #{description} but had attributes #{formatted_values}"
          end
        end

        def failure_message_when_negated
          respond_to_failure_message_or { "expected #{actual_formatted} not to #{description}" }
        end

        private

        def actual_formatted
          ObjectFormatter.format(@actual)
        end

        def perform_match(predicate)
          expected.__send__(predicate) do |attribute_key, attribute_value|
            actual_has_attribute?(attribute_key, attribute_value)
          end
        end

        def actual_has_attribute?(attribute_key, attribute_value)
          values_match?(attribute_value, @values.fetch(attribute_key))
        end

        def respond_to_attributes?
          matches = respond_to_matcher.matches?(@actual)
          @respond_to_failed = !matches
          cache_all_values if matches
          matches
        end

        def cache_all_values
          @values = {}
          expected.each_key do |attribute_key|
            @values[attribute_key] = @actual.__send__(attribute_key)
          end
        end

        def respond_to_matcher
          @respond_to_matcher ||= RespondTo.new(*expected.keys).with(0).arguments.tap { |matcher| matcher.instance_variable_set(:@expected_arity, 0) }
        end

        def respond_to_failure_message_or
          if respond_to_failed
            respond_to_matcher.failure_message.sub("with 0 arguments", "").rstrip
          else
            yield
          end
        end

        def formatted_values
          values = Composable.surface_descriptions_in(@values)
          ObjectFormatter.format(values)
        end
      end

      class Satisfy < BaseMatcher
        def initialize(description = nil, &block)
          @description = description
          @block = block
        end

        def matches?(actual, &block)
          @block = block if block
          @actual = actual
          @block.call(actual)
        end

        def description
          @description ||= "satisfy #{block_representation}"
        end

        def failure_message
          "expected #{actual_formatted} to #{description}"
        end

        def failure_message_when_negated
          "expected #{actual_formatted} not to #{description}"
        end

        private

        def block_representation
          snippet = BlockSnippet.for(@block, "satisfy")
          snippet ? "expression `#{snippet}`" : "block"
        end
      end

      class Change < BaseMatcher
        def by(expected_delta)
          ChangeRelatively.new(change_details, expected_delta, :by) do |actual_delta|
            values_match?(expected_delta, actual_delta)
          end
        end

        def by_at_least(minimum)
          ChangeRelatively.new(change_details, minimum, :by_at_least) do |actual_delta|
            actual_delta >= minimum
          end
        end

        def by_at_most(maximum)
          ChangeRelatively.new(change_details, maximum, :by_at_most) do |actual_delta|
            actual_delta <= maximum
          end
        end

        def to(value)
          ChangeToValue.new(change_details, value)
        end

        def from(value)
          ChangeFromValue.new(change_details, value)
        end

        def matches?(event_proc)
          raise_block_syntax_error if block_given?
          perform_change(event_proc) && change_details.changed?
        end

        def does_not_match?(event_proc)
          raise_block_syntax_error if block_given?
          perform_change(event_proc) && !change_details.changed?
        end

        def failure_message
          "expected #{change_details.value_representation} to have changed, " \
          "but #{positive_failure_reason}"
        end

        def failure_message_when_negated
          "expected #{change_details.value_representation} not to have changed, " \
          "but #{negative_failure_reason}"
        end

        def description
          "change #{change_details.value_representation}"
        end

        def supports_block_expectations?
          true
        end

        def supports_value_expectations?
          false
        end

        private

        def initialize(receiver=nil, message=nil, &block)
          @receiver = receiver
          @message = message
          @block = block
        end

        def change_details
          @change_details ||= ChangeDetails.new("change", @receiver, @message, &@block)
        end

        def perform_change(event_proc)
          @event_proc = event_proc
          change_details.perform_change(event_proc) do |actual_before|
            @actual_before_description = description_of(actual_before)
          end
        end

        def raise_block_syntax_error
          raise SyntaxError, "Block not received by the `change` matcher. " \
          "Perhaps you want to use `{ ... }` instead of do/end?"
        end

        def positive_failure_reason
          return "was not given a block" unless Proc === @event_proc
          "is still #{@actual_before_description}"
        end

        def negative_failure_reason
          return "was not given a block" unless Proc === @event_proc
          "did change from #{@actual_before_description} " \
          "to #{description_of change_details.actual_after}"
        end
      end

      class ChangeRelatively < BaseMatcher
        def initialize(change_details, expected_delta, relativity, &comparer)
          @change_details = change_details
          @expected_delta = expected_delta
          @relativity     = relativity
          @comparer       = comparer
        end

        def failure_message
          "expected #{@change_details.value_representation} to have changed " \
          "#{@relativity.to_s.tr('_', ' ')} " \
          "#{description_of @expected_delta}, but #{failure_reason}"
        end

        def matches?(event_proc)
          @event_proc = event_proc
          @change_details.perform_change(event_proc) && @comparer.call(@change_details.actual_delta)
        end

        def does_not_match?(_event_proc)
          raise NotImplementedError, "`expect { }.not_to change " \
            "{ }.#{@relativity}()` is not supported"
        end

        def description
          "change #{@change_details.value_representation} " \
          "#{@relativity.to_s.tr('_', ' ')} #{description_of @expected_delta}"
        end

        def supports_block_expectations?
          true
        end

        def supports_value_expectations?
          false
        end

        private

        def failure_reason
          return "was not given a block" unless Proc === @event_proc
          "was changed by #{description_of @change_details.actual_delta}"
        end
      end

      class SpecificValuesChange < BaseMatcher
        # Every object is a BasicObject, so this matches any value.
        MATCH_ANYTHING = ::Object.ancestors.last

        def initialize(change_details, from, to)
          @change_details  = change_details
          @expected_before = from
          @expected_after  = to
        end

        def matches?(event_proc)
          perform_change(event_proc) && @change_details.changed? && @matches_before && matches_after?
        end

        def description
          "change #{@change_details.value_representation} #{change_description}"
        end

        def failure_message
          return not_given_a_block_failure unless Proc === @event_proc
          return before_value_failure      unless @matches_before
          return did_not_change_failure    unless @change_details.changed?
          after_value_failure
        end

        def supports_block_expectations?
          true
        end

        def supports_value_expectations?
          false
        end

        private

        def perform_change(event_proc)
          @event_proc = event_proc
          @change_details.perform_change(event_proc) do |actual_before|
            @matches_before = values_match?(@expected_before, actual_before)
            @actual_before_description = description_of(actual_before)
          end
        end

        def matches_after?
          values_match?(@expected_after, @change_details.actual_after)
        end

        def before_value_failure
          "expected #{@change_details.value_representation} " \
          "to have initially been #{description_of @expected_before}, " \
          "but was #{@actual_before_description}"
        end

        def after_value_failure
          "expected #{@change_details.value_representation} " \
          "to have changed to #{description_of @expected_after}, " \
          "but is now #{description_of @change_details.actual_after}"
        end

        def did_not_change_failure
          "expected #{@change_details.value_representation} " \
          "to have changed #{change_description}, but did not change"
        end

        def did_change_failure
          "expected #{@change_details.value_representation} not to have changed, but " \
          "did change from #{@actual_before_description} " \
          "to #{description_of @change_details.actual_after}"
        end

        def not_given_a_block_failure
          "expected #{@change_details.value_representation} to have changed " \
          "#{change_description}, but was not given a block"
        end
      end

      class ChangeFromValue < SpecificValuesChange
        def initialize(change_details, expected_before)
          @description_suffix = nil
          super(change_details, expected_before, MATCH_ANYTHING)
        end

        def to(value)
          @expected_after     = value
          @description_suffix = " to #{description_of value}"
          self
        end

        def does_not_match?(event_proc)
          if @description_suffix
            raise NotImplementedError, "`expect { }.not_to change { }.to()` " \
              "is not supported"
          end

          perform_change(event_proc) && !@change_details.changed? && @matches_before
        end

        def failure_message_when_negated
          return not_given_a_block_failure unless Proc === @event_proc
          return before_value_failure unless @matches_before
          did_change_failure
        end

        private

        def change_description
          "from #{description_of @expected_before}#{@description_suffix}"
        end
      end

      class ChangeToValue < SpecificValuesChange
        def initialize(change_details, expected_after)
          @description_suffix = nil
          super(change_details, MATCH_ANYTHING, expected_after)
        end

        def from(value)
          @expected_before    = value
          @description_suffix = " from #{description_of value}"
          self
        end

        def does_not_match?(_event_proc)
          raise NotImplementedError, "`expect { }.not_to change { }.to()` " \
            "is not supported"
        end

        private

        def change_description
          "to #{description_of @expected_after}#{@description_suffix}"
        end
      end

      class ChangeDetails
        attr_reader :actual_after

        UNDEFINED = Module.new.freeze

        def initialize(matcher_name, receiver=nil, message=nil, &block)
          if receiver && !message
            raise(
              ArgumentError,
              "`change` requires either an object and message " \
              "(`change(obj, :msg)`) or a block (`change { }`). " \
              "You passed an object but no message."
            )
          end

          @matcher_name = matcher_name
          @receiver = receiver
          @message = message
          @value_proc = block
          @actual_after = UNDEFINED
        end

        def value_representation
          @value_representation ||=
            if @message
              "`#{message_notation(@receiver, @message)}`"
            elsif (value_block_snippet = extract_value_block_snippet)
              "`#{value_block_snippet}`"
            else
              'result'
            end
        end

        def perform_change(event_proc)
          @actual_before = evaluate_value_proc
          @before_hash = @actual_before.hash
          yield @actual_before if block_given?

          return false unless Proc === event_proc
          event_proc.call

          @actual_after = evaluate_value_proc
          @actual_hash = @actual_after.hash
          true
        end

        def changed?
          @actual_before != @actual_after || @before_hash != @actual_hash
        end

        def actual_delta
          @actual_after - @actual_before
        end

        private

        def evaluate_value_proc
          @value_proc ? @value_proc.call : @receiver.__send__(@message)
        end

        def message_notation(receiver, message)
          case receiver
          when Module
            "#{receiver}.#{message}"
          else
            "#{receiver.class}##{message}"
          end
        end

        def extract_value_block_snippet
          return nil unless @value_proc

          BlockSnippet.for(@value_proc, @matcher_name)
        end
      end

      class Output < BaseMatcher
        def initialize(expected)
          @expected = expected
          @actual = ""
          @block = nil
          @stream_capturer = NullCapture
        end

        def matches?(block)
          @block = block
          return false unless Proc === block

          @actual = @stream_capturer.capture(block)
          @expected ? values_match?(@expected, @actual) : captured?
        end

        def does_not_match?(block)
          !matches?(block) && Proc === block
        end

        def to_stdout
          @stream_capturer = CaptureStdout
          self
        end

        def to_stderr
          @stream_capturer = CaptureStderr
          self
        end

        def to_stdout_from_any_process
          @stream_capturer = CaptureStreamToTempfile.new("stdout", $stdout)
          self
        end

        def to_stderr_from_any_process
          @stream_capturer = CaptureStreamToTempfile.new("stderr", $stderr)
          self
        end

        def failure_message
          "expected block to #{description}, but #{positive_failure_reason}"
        end

        def failure_message_when_negated
          "expected block to not #{description}, but #{negative_failure_reason}"
        end

        def description
          if @expected
            "output #{description_of @expected} to #{@stream_capturer.name}"
          else
            "output to #{@stream_capturer.name}"
          end
        end

        def diffable?
          true
        end

        def supports_block_expectations?
          true
        end

        def supports_value_expectations?
          false
        end

        private

        def captured?
          @actual.length > 0
        end

        def positive_failure_reason
          return "was not a block" unless Proc === @block
          return "output #{actual_output_description}" if @expected

          "did not"
        end

        def negative_failure_reason
          return "was not a block" unless Proc === @block

          "output #{actual_output_description}"
        end

        def actual_output_description
          return "nothing" unless captured?

          actual_formatted
        end
      end

      module NullCapture
        def self.name
          "some stream"
        end

        def self.capture(_block)
          raise "You must chain `to_stdout` or `to_stderr` off of the `output(...)` matcher."
        end
      end

      module CaptureStdout
        def self.name
          "stdout"
        end

        def self.capture(block)
          original = $stdout
          captured = StringIO.new
          $stdout = captured
          block.call
          captured.string
        ensure
          $stdout = original
        end
      end

      module CaptureStderr
        def self.name
          "stderr"
        end

        def self.capture(block)
          original = $stderr
          captured = StringIO.new
          $stderr = captured
          block.call
          captured.string
        ensure
          $stderr = original
        end
      end

      # Captures what any process writes to the stream, a child process
      # included, by pointing the stream's file descriptor at a temporary file
      # while the block runs.
      class CaptureStreamToTempfile
        attr_reader :name

        def initialize(name, stream)
          @name = name
          @stream = stream
        end

        def capture(block)
          require "tempfile"
          captured = Tempfile.new(@name)
          original = @stream.clone
          captured.sync = true
          @stream.flush
          @stream.reopen(captured)
          block.call
          @stream.flush
          captured.rewind
          captured.read
        ensure
          # The clone is left open: when the stream is a wrapper rather than
          # an IO, as a suite's stderr splitter is, closing the clone closes
          # the stream it wraps.
          @stream.reopen(original) if original
          captured&.close
          captured&.unlink
        end
      end

      # What `yield_control` and the other yield matchers hand the method
      # under test as its block, recording each yield.
      class YieldProbe
        def self.probe(block, &callback)
          probe = new(block, &callback)
          return probe unless probe.has_block?

          probe.probe
        end

        attr_accessor :num_yields, :yielded_args

        def initialize(block, &callback)
          @block = block
          @callback = callback || proc {}
          @used = false
          self.num_yields = 0
          self.yielded_args = []
        end

        def has_block?
          Proc === @block
        end

        def probe
          assert_valid_expect_block!
          @block.call(self)
          assert_used!
          self
        end

        def to_proc
          @used = true
          probe = self
          callback = @callback
          proc do |*args|
            probe.num_yields += 1
            probe.yielded_args << args
            callback.call(*args)
            nil
          end
        end

        def single_yield_args
          yielded_args.first
        end

        def yielded_once?(matcher_name)
          case num_yields
          when 1 then true
          when 0 then false
          else
            raise "The #{matcher_name} matcher is not designed to be used with a method that yields multiple times. Use the yield_successive_args matcher for that case."
          end
        end

        def assert_used!
          return if @used

          raise "You must pass the argument yielded to your expect block on to the method-under-test as a block. It acts as a probe that allows the matcher to detect whether or not the method-under-test yields, and, if so, how many times. See the yield matchers documentation for an example."
        end

        def assert_valid_expect_block!
          block_signature = @block.arity
          return if block_signature == 1

          raise "Your expect block must accept an argument to be used with this matcher. Pass the argument as a block on to the method you are testing."
        end
      end

      class YieldControl < BaseMatcher
        def initialize
          @expectation_type = @expected_yields_count = nil
        end

        def once
          exactly(1)
        end

        def twice
          exactly(2)
        end

        def thrice
          exactly(3)
        end

        def exactly(number)
          set_expected_yields_count(:==, number)
          self
        end

        def at_most(number)
          set_expected_yields_count(:<=, number)
          self
        end

        def at_least(number)
          set_expected_yields_count(:>=, number)
          self
        end

        def times
          self
        end

        def matches?(block)
          @probe = YieldProbe.probe(block)
          return false unless @probe.has_block?

          return @probe.num_yields > 0 unless @expectation_type

          @probe.num_yields.__send__(@expectation_type, @expected_yields_count)
        end

        def does_not_match?(block)
          !matches?(block) && @probe.has_block?
        end

        def failure_message
          "expected given block to yield control#{failure_reason}"
        end

        def failure_message_when_negated
          "expected given block not to yield control#{failure_reason}"
        end

        def description
          "yield control#{relativity_count}"
        end

        def supports_block_expectations?
          true
        end

        def supports_value_expectations?
          false
        end

        private

        def set_expected_yields_count(relativity, number)
          raise "Multiple count constraints are not supported" if @expectation_type

          @expectation_type = relativity
          @expected_yields_count = case number
                                   when Numeric then number
                                   when :once then 1
                                   when :twice then 2
                                   when :thrice then 3
                                   end
        end

        def failure_reason
          return " but was not a block" unless @probe.has_block?
          return " but did not yield" unless @expected_yields_count

          " #{human_readable_expectation_type}#{human_readable_count(@expected_yields_count)}" \
            " but yielded #{human_readable_count(@probe.num_yields)}"
        end

        def relativity_count
          ""
        end

        def human_readable_expectation_type
          case @expectation_type
          when :<= then "at most "
          when :>= then "at least "
          else ""
          end
        end

        def human_readable_count(count)
          case count
          when 1 then "once"
          when 2 then "twice"
          else "#{count} times"
          end
        end
      end

      class YieldWithNoArgs < BaseMatcher
        def matches?(block)
          @probe = YieldProbe.probe(block)
          return false unless @probe.has_block?

          @probe.yielded_once?(:yield_with_no_args) && @probe.single_yield_args.empty?
        end

        def does_not_match?(block)
          !matches?(block) && @probe.has_block?
        end

        def failure_message
          "expected given block to yield with no arguments, but #{positive_failure_reason}"
        end

        def failure_message_when_negated
          "expected given block not to yield with no arguments, but #{negative_failure_reason}"
        end

        def supports_block_expectations?
          true
        end

        def supports_value_expectations?
          false
        end

        private

        def positive_failure_reason
          return "was not a block" unless @probe.has_block?
          return "did not yield" if @probe.num_yields.zero?

          "yielded with arguments: #{description_of @probe.single_yield_args}"
        end

        def negative_failure_reason
          return "was not a block" unless @probe.has_block?

          "did"
        end
      end

      class YieldWithArgs < BaseMatcher
        def initialize(*args)
          @expected = args
        end

        def matches?(block)
          @args_matched_when_yielded = true
          @probe = YieldProbe.new(block) do
            @actual = @probe.single_yield_args
            @actual_formatted = actual_formatted
            @args_matched_when_yielded &&= args_currently_match?
          end
          return false unless @probe.has_block?

          @probe.probe
          @probe.yielded_once?(:yield_with_args) && @args_matched_when_yielded
        end

        def does_not_match?(block)
          !matches?(block) && @probe.has_block?
        end

        def failure_message
          "expected given block to yield with arguments, but #{positive_failure_reason}"
        end

        def failure_message_when_negated
          "expected given block not to yield with arguments, but #{negative_failure_reason}"
        end

        def description
          desc = "yield with args"
          desc = "#{desc}(#{expected_arg_description})" unless @expected.empty?
          desc
        end

        def supports_block_expectations?
          true
        end

        def supports_value_expectations?
          false
        end

        private

        def positive_failure_reason
          return "was not a block" unless @probe.has_block?
          return "did not yield" if @probe.num_yields.zero?

          @positive_args_failure
        end

        def expected_arg_description
          @expected.map { |arg| description_of arg }.join(", ")
        end

        def negative_failure_reason
          if !@probe.has_block?
            "was not a block"
          elsif @args_matched_when_yielded && !@expected.empty?
            "yielded with expected arguments" \
              "\nexpected not: #{surface_descriptions_in(@expected).inspect}" \
              "\n         got: #{@actual_formatted}"
          else
            "did"
          end
        end

        def args_currently_match?
          if @expected.empty?
            @positive_args_failure = "yielded with no arguments" if @actual.empty?
            return !@actual.empty?
          end

          unless (match = all_args_match?)
            @positive_args_failure = "yielded with unexpected arguments" \
                                     "\nexpected: #{surface_descriptions_in(@expected).inspect}" \
                                     "\n     got: #{@actual_formatted}"
          end
          match
        end

        def all_args_match?
          values_match?(@expected, @actual)
        end
      end

      class YieldSuccessiveArgs < BaseMatcher
        def initialize(*args)
          @expected = args
        end

        def matches?(block)
          @actual_formatted = []
          @actual = []
          args_matched_when_yielded = true
          yield_count = 0
          @probe = YieldProbe.probe(block) do |*arg_array|
            arg_or_args = arg_array.size == 1 ? arg_array.first : arg_array
            @actual_formatted << ObjectFormatter.format(arg_or_args)
            @actual << arg_or_args
            args_matched_when_yielded &&= values_match?(@expected[yield_count], arg_or_args)
            yield_count += 1
          end
          return false unless @probe.has_block?

          args_matched_when_yielded && yield_count == @expected.length
        end

        def does_not_match?(block)
          !matches?(block) && @probe.has_block?
        end

        def failure_message
          "expected given block to yield successively with arguments, but #{positive_failure_reason}"
        end

        def failure_message_when_negated
          "expected given block not to yield successively with arguments, but #{negative_failure_reason}"
        end

        def description
          "yield successive args(#{expected_arg_description})"
        end

        def supports_block_expectations?
          true
        end

        def supports_value_expectations?
          false
        end

        private

        def expected_arg_description
          @expected.map { |arg| description_of arg }.join(", ")
        end

        def positive_failure_reason
          return "was not a block" unless @probe.has_block?

          "yielded with unexpected arguments" \
            "\nexpected: #{surface_descriptions_in(@expected).inspect}" \
            "\n     got: [#{@actual_formatted.join(', ')}]"
        end

        def negative_failure_reason
          return "was not a block" unless @probe.has_block?

          "yielded with expected arguments" \
            "\nexpected not: #{surface_descriptions_in(@expected).inspect}" \
            "\n         got: [#{@actual_formatted.join(', ')}]"
        end
      end

      # `matcher.and(other)` and `matcher.or(other)`.
      class Compound < BaseMatcher
        attr_reader :matcher_1, :matcher_2, :evaluator

        def initialize(matcher_1, matcher_2)
          @matcher_1 = matcher_1
          @matcher_2 = matcher_2
        end

        def does_not_match?(_actual)
          raise NotImplementedError, "`expect(...).not_to matcher.#{conjunction} matcher` " \
                                     "is not supported, since it creates a bit of an ambiguity. Instead, define negated versions " \
                                     "of whatever matchers you wish to negate with `RSpec::Matchers.define_negated_matcher` and " \
                                     "use `expect(...).to matcher.#{conjunction} matcher`."
        end

        def description
          "#{matcher_1.description} #{conjunction} #{matcher_2.description}"
        end

        def supports_block_expectations?
          matcher_supports_block_expectations?(matcher_1) &&
            matcher_supports_block_expectations?(matcher_2)
        end

        def supports_value_expectations?
          matcher_supports_value_expectations?(matcher_1) &&
            matcher_supports_value_expectations?(matcher_2)
        end

        def expects_call_stack_jump?
          NestedEvaluator.matcher_expects_call_stack_jump?(matcher_1) ||
            NestedEvaluator.matcher_expects_call_stack_jump?(matcher_2)
        end

        def diffable?
          matcher_is_diffable?(matcher_1) || matcher_is_diffable?(matcher_2)
        end

        def expected
          return nil unless evaluator

          ::Rex::Matchers::MultiMatcherDiff.for_many_matchers(diffable_matcher_list)
        end

        protected

        def diffable_matcher_list
          list = []
          list.concat(diffable_matcher_list_for(matcher_1)) unless matcher_1_matches?
          list.concat(diffable_matcher_list_for(matcher_2)) unless matcher_2_matches?
          list
        end

        private

        def initialize_copy(other)
          @matcher_1 = @matcher_1.clone
          @matcher_2 = @matcher_2.clone
          super
        end

        def match(_expected, actual)
          evaluator_class =
            if supports_block_expectations? && Proc === actual
              NestedEvaluator
            elsif supports_value_expectations?
              SequentialEvaluator
            else
              raise "Block and value matchers can't be combined in a compound expectation " \
                    "(#{matcher_1.description}, #{matcher_2.description})"
            end
          @evaluator = evaluator_class.new(actual, matcher_1, matcher_2)
        end

        def indent_multiline_message(message)
          message.lines.map do |line|
            line =~ /\S/ ? "   " + line : line
          end.join
        end

        def compound_failure_message
          "#{indent_multiline_message(matcher_1.failure_message.sub(/\n+\z/, ''))}" \
            "\n\n...#{conjunction}:" \
            "\n\n#{indent_multiline_message(matcher_2.failure_message.sub(/\A\n+/, ''))}"
        end

        def matcher_1_matches?
          evaluator.matcher_matches?(matcher_1)
        end

        def matcher_2_matches?
          evaluator.matcher_matches?(matcher_2)
        end

        def matcher_supports_block_expectations?(matcher)
          matcher.supports_block_expectations?
        rescue NoMethodError
          false
        end

        def matcher_supports_value_expectations?(matcher)
          matcher.supports_value_expectations?
        rescue NoMethodError
          true
        end

        def matcher_is_diffable?(matcher)
          matcher.diffable?
        rescue NoMethodError
          false
        end

        def diffable_matcher_list_for(matcher)
          return [] unless matcher_is_diffable?(matcher)
          return matcher.diffable_matcher_list if Compound === matcher

          [matcher]
        end

        # Matches each matcher against a value in turn.
        class SequentialEvaluator
          def initialize(actual, *)
            @actual = actual
          end

          def matcher_matches?(matcher)
            matcher.matches?(@actual)
          end
        end

        # Matches two block matchers with one call of the block, one matcher
        # wrapped around the other.
        class NestedEvaluator
          def initialize(actual, matcher_1, matcher_2)
            @actual = actual
            @matcher_1 = matcher_1
            @matcher_2 = matcher_2
            @match_results = {}

            inner, outer = order_block_matchers

            @match_results[outer] = outer.matches?(proc do |*arguments|
              @match_results[inner] = inner.matches?(inner_matcher_block(arguments))
            end)
          end

          def matcher_matches?(matcher)
            @match_results.fetch(matcher) do
              raise ArgumentError, "Your #{matcher.description} has no match " \
                                   "results, this can occur when an unexpected call stack or " \
                                   "local jump occurs. Perhaps one of your matchers needs to " \
                                   "declare `expects_call_stack_jump?` as `true`?"
            end
          end

          def self.matcher_expects_call_stack_jump?(matcher)
            matcher.expects_call_stack_jump?
          rescue NoMethodError
            false
          end

          private

          # The inner matcher is handed the value the outer block yields
          # with, or a block that passes on what the outer one yields.
          def inner_matcher_block(outer_arguments)
            return @actual if outer_arguments.empty?

            proc do |*inner_arguments|
              unless inner_arguments.empty?
                raise ArgumentError, "(#{@matcher_1.description}) and " \
                                     "(#{@matcher_2.description}) cannot be combined in a compound expectation " \
                                     "since both matchers pass arguments to the block."
              end

              @actual.call(*outer_arguments)
            end
          end

          def order_block_matchers
            return @matcher_1, @matcher_2 unless self.class.matcher_expects_call_stack_jump?(@matcher_2)
            return @matcher_2, @matcher_1 unless self.class.matcher_expects_call_stack_jump?(@matcher_1)

            raise ArgumentError, "(#{@matcher_1.description}) and " \
                                 "(#{@matcher_2.description}) cannot be combined in a compound expectation " \
                                 "because they both expect a call stack jump."
          end
        end

        class And < self
          def failure_message
            if matcher_1_matches?
              matcher_2.failure_message
            elsif matcher_2_matches?
              matcher_1.failure_message
            else
              compound_failure_message
            end
          end

          private

          def match(*)
            super
            matcher_1_matches? && matcher_2_matches?
          end

          def conjunction
            "and"
          end
        end

        class Or < self
          def failure_message
            compound_failure_message
          end

          private

          def match(*)
            super
            matcher_1_matches? || matcher_2_matches?
          end

          def conjunction
            "or"
          end
        end
      end

      class OperatorMatcher
        OPERATORS = %i[== === =~ > >= < <=].freeze

        def initialize(actual)
          @actual = actual
        end

        OPERATORS.each do |operator|
          define_method(operator) do |expected|
            eval_match(@actual, operator, expected)
          end
        end

        def !=(_expected)
          raise ArgumentError, "`!=` is not supported"
        end

        def description
          "#{@operator} #{ObjectFormatter.format(@expected)}"
        end

        def fail_with_message(message)
          Rex::Expectations.fail_with(message, @expected, @actual)
        end
      end

      class PositiveOperatorMatcher < OperatorMatcher
        def eval_match(actual, operator, expected)
          @operator = operator
          @expected = expected
          return true if actual.__send__(operator, expected)

          if operator == :=~
            fail_with_message("expected: #{ObjectFormatter.format(expected)}\n     got: #{ObjectFormatter.format(actual)} (using =~)")
          else
            fail_with_message("expected: #{operator} #{ObjectFormatter.format(expected)}\n     got: #{operator.to_s.gsub(/./, ' ')} #{ObjectFormatter.format(actual)}")
          end
        end
      end

      class NegativeOperatorMatcher < OperatorMatcher
        def eval_match(actual, operator, expected)
          @operator = operator
          @expected = expected
          return true unless actual.__send__(operator, expected)

          fail_with_message("expected not: #{operator} #{ObjectFormatter.format(expected)}\n         got: #{operator.to_s.gsub(/./, ' ')} #{ObjectFormatter.format(actual)}")
        end
      end
    end

    # A matcher reached through an alias, which describes itself with the
    # alias's name.
    # A class with as few methods as Ruby allows, so a delegator can hand
    # nearly every message on through `method_missing`.
    class BaseDelegator
      kept_methods = [
        # Methods that raise warnings if removed.
        :__id__, :__send__, :object_id,

        # Methods that are explicitly undefined in some subclasses.
        :==, :===,

        # Methods kept on purpose.
        :class, :respond_to?, :__method__, :method, :dup,
        :clone, :initialize_dup, :initialize_copy, :initialize_clone
      ]
      instance_methods.each do |method|
        undef_method(method) unless kept_methods.include?(method.to_sym)
      end
    end

    # Wraps a matcher, handing it every message the wrapper does not answer.
    class MatcherDelegator < BaseDelegator
      include Composable
      attr_reader :base_matcher

      def initialize(base_matcher)
        @base_matcher = base_matcher
      end

      def method_missing(*arguments, &block)
        base_matcher.__send__(*arguments, &block)
      end
      ruby2_keywords(:method_missing) if respond_to?(:ruby2_keywords, true)

      def respond_to_missing?(name, include_all = false)
        super || base_matcher.respond_to?(name, include_all)
      end

      def initialize_copy(other)
        @base_matcher = @base_matcher.clone
        super
      end
    end

    # What `alias_matcher` builds: the matcher it names, described with the
    # alias's own phrasing.
    class AliasedMatcher < MatcherDelegator
      def initialize(base_matcher, description_block)
        @description_block = description_block
        super(base_matcher)
      end

      def method_missing(*)
        return_value = super
        return return_value unless Rex::Matchers.is_a_matcher?(return_value)

        self.class.new(return_value, @description_block)
      end

      def description
        @description_block.call(super)
      end

      def failure_message
        @description_block.call(super)
      end

      def failure_message_when_negated
        @description_block.call(super)
      end
    end

    # An alias whose base matcher answers `==` and `===` itself.
    class AliasedMatcherWithOperatorSupport < AliasedMatcher
      undef ==
      undef ===
    end

    # What `define_negated_matcher` builds.
    class AliasedNegatedMatcher < AliasedMatcher
      def matches?(*arguments, &block)
        if @base_matcher.respond_to?(:does_not_match?)
          @base_matcher.does_not_match?(*arguments, &block)
        else
          !super
        end
      end

      def does_not_match?(*arguments, &block)
        @base_matcher.matches?(*arguments, &block)
      end

      def failure_message
        optimal_failure_message(__method__, :failure_message_when_negated)
      end

      def failure_message_when_negated
        optimal_failure_message(__method__, :failure_message)
      end

      private

      DefaultFailureMessages = BuiltIn::BaseMatcher::DefaultFailureMessages

      # A base matcher with RSpec's default messages has its own message
      # rephrased; one with custom messages has the inverted one used.
      def optimal_failure_message(same, inverted)
        if DefaultFailureMessages.has_default_failure_messages?(@base_matcher)
          base_message = @base_matcher.__send__(same)
          overridden = @description_block.call(base_message)
          return overridden if overridden != base_message
        end

        @base_matcher.__send__(inverted)
      end
    end

    # The matcher methods every example can call.
    def eq(expected)
      BuiltIn::Eq.new(expected)
    end
    alias_matcher :an_object_eq_to, :eq
    alias_matcher :eq_to, :eq

    def eql(expected)
      BuiltIn::Eql.new(expected)
    end
    alias_matcher :an_object_eql_to, :eql
    alias_matcher :eql_to, :eql

    def equal(expected)
      BuiltIn::Equal.new(expected)
    end
    alias_matcher :an_object_equal_to, :equal
    alias_matcher :equal_to, :equal

    def be(*arguments)
      arguments.empty? ? BuiltIn::Be.new : equal(*arguments)
    end
    alias_matcher :a_value, :be, klass: AliasedMatcherWithOperatorSupport

    def be_truthy
      BuiltIn::BeTruthy.new
    end
    alias_matcher :a_truthy_value, :be_truthy

    def be_falsey
      BuiltIn::BeFalsey.new
    end
    alias_matcher :be_falsy, :be_falsey
    alias_matcher :a_falsey_value, :be_falsey
    alias_matcher :a_falsy_value, :be_falsey

    def be_nil
      BuiltIn::BeNil.new
    end
    alias_matcher :a_nil_value, :be_nil

    def be_a(klass)
      be_a_kind_of(klass)
    end
    alias be_an be_a

    def be_a_kind_of(expected)
      BuiltIn::BeAKindOf.new(expected)
    end
    alias be_kind_of be_a_kind_of
    alias_matcher :a_kind_of, :be_a_kind_of

    def be_an_instance_of(expected)
      BuiltIn::BeAnInstanceOf.new(expected)
    end
    alias be_instance_of be_an_instance_of
    alias_matcher :an_instance_of, :be_an_instance_of

    def be_between(min, max)
      BuiltIn::BeBetween.new(min, max)
    end
    alias_matcher :a_value_between, :be_between

    def be_within(delta)
      BuiltIn::BeWithin.new(delta)
    end
    alias_matcher :a_value_within, :be_within
    alias_matcher :within, :be_within

    def change(receiver = nil, message = nil, &block)
      BuiltIn::Change.new(receiver, message, &block)
    end
    alias_matcher :a_block_changing, :change
    alias_matcher :changing, :change

    def contain_exactly(*items)
      BuiltIn::ContainExactly.new(items)
    end
    alias_matcher :a_collection_containing_exactly, :contain_exactly
    alias_matcher :containing_exactly, :contain_exactly

    def match_array(items)
      contain_exactly(*items)
    end
    alias_matcher :an_array_matching, :match_array do |description|
      description.sub("contain exactly", "an array containing exactly")
    end

    def cover(*values)
      BuiltIn::Cover.new(*values)
    end

    def end_with(*expected)
      BuiltIn::EndWith.new(*expected)
    end
    alias_matcher :a_collection_ending_with, :end_with
    alias_matcher :a_string_ending_with, :end_with
    alias_matcher :ending_with, :end_with

    def start_with(*expected)
      BuiltIn::StartWith.new(*expected)
    end
    alias_matcher :a_collection_starting_with, :start_with
    alias_matcher :a_string_starting_with, :start_with
    alias_matcher :starting_with, :start_with

    def exist(*arguments)
      BuiltIn::Exist.new(*arguments)
    end
    alias_matcher :an_object_existing, :exist
    alias_matcher :existing, :exist

    def have_attributes(expected)
      BuiltIn::HaveAttributes.new(expected)
    end
    alias_matcher :an_object_having_attributes, :have_attributes
    alias_matcher :having_attributes, :have_attributes

    def include(*expected)
      BuiltIn::Include.new(*expected)
    end
    alias_matcher :a_collection_including, :include
    alias_matcher :a_string_including, :include
    alias_matcher :a_hash_including, :include
    alias_matcher :including, :include

    def all(expected)
      BuiltIn::All.new(expected)
    end

    def match(expected)
      BuiltIn::Match.new(expected)
    end
    alias_matcher :match_regex, :match
    alias_matcher :an_object_matching, :match
    alias_matcher :a_string_matching, :match
    alias_matcher :matching, :match

    def output(expected = nil)
      BuiltIn::Output.new(expected)
    end
    alias_matcher :a_block_outputting, :output

    def raise_error(error = BuiltIn::RaiseError::UndefinedValue, message = nil, &block)
      BuiltIn::RaiseError.new(error, message, &block)
    end
    alias raise_exception raise_error
    alias_matcher :a_block_raising, :raise_error do |description|
      description.sub("raise", "a block raising")
    end
    alias_matcher :raising, :raise_error do |description|
      description.sub("raise", "raising")
    end

    def respond_to(*names)
      BuiltIn::RespondTo.new(*names)
    end
    alias_matcher :an_object_responding_to, :respond_to
    alias_matcher :responding_to, :respond_to

    def satisfy(description = nil, &block)
      BuiltIn::Satisfy.new(description, &block)
    end
    alias_matcher :an_object_satisfying, :satisfy
    alias_matcher :satisfying, :satisfy

    def throw_symbol(expected_symbol = nil, expected_arg = nil)
      BuiltIn::ThrowSymbol.new(expected_symbol, expected_arg)
    end
    alias_matcher :a_block_throwing, :throw_symbol do |description|
      description.sub("throw", "a block throwing")
    end
    alias_matcher :throwing, :throw_symbol do |description|
      description.sub("throw", "throwing")
    end

    def yield_control
      BuiltIn::YieldControl.new
    end
    alias_matcher :a_block_yielding_control, :yield_control
    alias_matcher :yielding_control, :yield_control

    def yield_with_no_args
      BuiltIn::YieldWithNoArgs.new
    end
    alias_matcher :a_block_yielding_with_no_args, :yield_with_no_args
    alias_matcher :yielding_with_no_args, :yield_with_no_args

    def yield_with_args(*args)
      BuiltIn::YieldWithArgs.new(*args)
    end
    alias_matcher :a_block_yielding_with_args, :yield_with_args
    alias_matcher :yielding_with_args, :yield_with_args

    def yield_successive_args(*args)
      BuiltIn::YieldSuccessiveArgs.new(*args)
    end
    alias_matcher :a_block_yielding_successive_args, :yield_successive_args
    alias_matcher :yielding_successive_args, :yield_successive_args

    def aggregate_failures(label = nil, metadata = {}, &block)
      Rex::Expectations::FailureAggregator.new(label, metadata).aggregate(&block)
    end

    def expect(value = Expectations::ExpectationTarget::UNDEFINED, &block)
      target = Expectations::ExpectationTarget.for(value, block)
      target.source_location = caller_locations(1, 1).first if Rex.configuration.source_expressions
      target
    end

    BE_PREDICATE_REGEX = /\Abe_(?:an?_)?(.*)/
    HAS_REGEX = /\Ahave_(.*)/

    def method_missing(name, *arguments, &block)
      case name.to_s
      when BE_PREDICATE_REGEX
        BuiltIn::BePredicate.new(name, *arguments, &block)
      when HAS_REGEX
        BuiltIn::Has.new(name, *arguments, &block)
      else
        super
      end
    end
    ruby2_keywords(:method_missing) if respond_to?(:ruby2_keywords, true)

    def respond_to_missing?(name, include_private = false)
      BE_PREDICATE_REGEX.match?(name.to_s) || HAS_REGEX.match?(name.to_s) || super
    end
  end
end

require "stringio"
require "rex/matchers/block_snippet"
require "rex/matchers/extras"
require "rex/matchers/dsl"
require "rex/matchers/differ"
require "rex/expectations/failure_aggregator"
