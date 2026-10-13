module Rex
  module Matchers
    module BuiltIn
      class Cover < BaseMatcher
        def initialize(*expected)
          @expected = expected
        end

        def matches?(range)
          @actual = range
          @expected.all? { |value| range.cover?(value) }
        end

        def does_not_match?(range)
          @actual = range
          expected.none? { |value| range.cover?(value) }
        end

        def description
          "cover#{EnglishPhrasing.list(@expected)}"
        end
      end

      class Exist < BaseMatcher
        def initialize(*expected)
          @expected = expected
        end

        def matches?(actual)
          @actual = actual
          @test = ExistenceTest.new(@actual, @expected)
          @test.valid_test? && @test.actual_exists?
        end

        def does_not_match?(actual)
          @actual = actual
          @test = ExistenceTest.new(@actual, @expected)
          @test.valid_test? && !@test.actual_exists?
        end

        def failure_message
          "expected #{actual_formatted} to exist#{@test.validity_message}"
        end

        def failure_message_when_negated
          "expected #{actual_formatted} not to exist#{@test.validity_message}"
        end

        class ExistenceTest < Struct.new(:actual, :expected)
          def valid_test?
            uniq_truthy_values.size == 1
          end

          def actual_exists?
            existence_values.first
          end

          def validity_message
            case uniq_truthy_values.size
            when 0
              " but it does not respond to either `exist?` or `exists?`"
            when 2
              " but `exist?` and `exists?` returned different values:\n\n" \
                " exist?: #{existence_values.first}\n" \
                "exists?: #{existence_values.last}"
            else
              ""
            end
          end

          private

          def uniq_truthy_values
            @uniq_truthy_values ||= existence_values.map { |value| !!value }.uniq
          end

          def existence_values
            @existence_values ||= predicates.map { |predicate| actual.__send__(predicate, *expected) }
          end

          def predicates
            @predicates ||= %i[exist? exists?].select { |predicate| actual.respond_to?(predicate) }
          end
        end
      end
    end
  end

  module Core
    # What a method accepts, read from `Method#parameters`, so a matcher or a
    # verifying double can say whether a call would be accepted.
    class MethodSignature
      attr_reader :min_non_kw_args, :max_non_kw_args, :required_kw_args, :optional_kw_args

      def initialize(method)
        @method = method
        @min_non_kw_args = 0
        @max_non_kw_args = 0
        @required_kw_args = []
        @optional_kw_args = []
        @any_keywords = false
        @rest = false
        method.parameters.each do |type, name|
          case type
          when :req then @min_non_kw_args += 1
                         @max_non_kw_args += 1
          when :opt then @max_non_kw_args += 1
          when :rest then @rest = true
          when :keyreq then @required_kw_args << name
          when :key then @optional_kw_args << name
          when :keyrest then @any_keywords = true
          end
        end
        @max_non_kw_args = Float::INFINITY if @rest
      end

      def accepts?(arity, keywords = [], unlimited = false, any_keywords = false)
        return false if unlimited && @max_non_kw_args != Float::INFINITY
        return false if any_keywords && !@any_keywords
        return false if arity && !(arity >= @min_non_kw_args && arity <= @max_non_kw_args)
        return true if @any_keywords

        missing = @required_kw_args - keywords
        unknown = keywords - @required_kw_args - @optional_kw_args
        missing.empty? && unknown.empty?
      end

      def accepts_keywords?
        @any_keywords || !@required_kw_args.empty? || !@optional_kw_args.empty?
      end

      def description
        parts = []
        parts << "arity of #{non_kw_args_arity_description}" unless non_kw_args_arity_description == "0"
        parts << "optional keyword args (#{@optional_kw_args.map(&:inspect).join(', ')})" unless @optional_kw_args.empty?
        parts << "required keyword args (#{@required_kw_args.map(&:inspect).join(', ')})" unless @required_kw_args.empty?
        parts << "any additional keyword args" if @any_keywords
        parts.join(" and ")
      end
    end
  end
end
