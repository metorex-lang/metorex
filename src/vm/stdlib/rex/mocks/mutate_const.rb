module Rex
  module Mocks
    # What `stub_const` and `hide_const` did to a constant, as
    # `RSpec::Mocks::Constant.original(name)` reports it.
    class Constant
      extend Support::RecursiveConstMethods

      def initialize(name)
        @name = name
        @previously_defined = false
        @stubbed = false
        @hidden = false
        @valid_name = true
        yield self if block_given?
      end

      attr_reader :name
      attr_accessor :original_value
      attr_writer :previously_defined, :stubbed, :hidden, :valid_name

      def previously_defined?
        @previously_defined
      end

      def mutated?
        @stubbed || @hidden
      end

      def stubbed?
        @stubbed
      end

      def hidden?
        @hidden
      end

      def valid_name?
        @valid_name
      end

      def to_s
        "#<#{self.class.name} #{name}>"
      end
      alias inspect to_s

      def self.unmutated(name)
        previously_defined = !!recursive_const_defined?(name)
      rescue NameError
        new(name) do |constant|
          constant.valid_name = false
        end
      else
        new(name) do |constant|
          constant.previously_defined = previously_defined
          constant.original_value = recursive_const_get(name) if previously_defined
        end
      end

      def self.original(name)
        mutator = ::Rex::Mocks.space.constant_mutator_for(name)
        mutator ? mutator.to_constant : unmutated(name)
      end
    end

    # Stubs and hides constants, and puts them back when the example ends.
    class ConstantMutator
      extend Support::RecursiveConstMethods

      def self.stub(constant_name, value, options = {})
        unless String === constant_name
          raise ArgumentError, "`stub_const` requires a String, but you provided a #{constant_name.class.name}"
        end

        mutator = if recursive_const_defined?(constant_name, &raise_on_invalid_const)
                    DefinedConstantReplacer
                  else
                    UndefinedConstantSetter
                  end

        mutate(mutator.new(constant_name, value, options[:transfer_nested_constants]))
        value
      end

      def self.hide(constant_name)
        mutate(ConstantHider.new(constant_name, nil, {}))
        nil
      end

      class BaseMutator
        include Support::RecursiveConstMethods

        attr_reader :original_value, :full_constant_name

        def initialize(full_constant_name, mutated_value, transfer_nested_constants)
          @full_constant_name = normalize_const_name(full_constant_name)
          @mutated_value = mutated_value
          @transfer_nested_constants = transfer_nested_constants
          @context_parts = @full_constant_name.split("::")
          @const_name = @context_parts.pop
          @reset_performed = false
        end

        def to_constant
          constant = Constant.new(full_constant_name)
          constant.original_value = original_value
          constant
        end

        def idempotently_reset
          reset unless @reset_performed
          @reset_performed = true
        end
      end

      class ConstantHider < BaseMutator
        def mutate
          return unless (@defined = recursive_const_defined?(full_constant_name))

          @context = recursive_const_get(@context_parts.join("::"))
          @original_value = get_const_defined_on(@context, @const_name)

          @context.__send__(:remove_const, @const_name)
        end

        def to_constant
          return Constant.unmutated(full_constant_name) unless @defined

          constant = super
          constant.hidden = true
          constant.previously_defined = true

          constant
        end

        def reset
          return unless @defined

          @context.const_set(@const_name, @original_value)
        end
      end

      class DefinedConstantReplacer < BaseMutator
        def initialize(*args)
          super
          @constants_to_transfer = []
        end

        def mutate
          @context = recursive_const_get(@context_parts.join("::"))
          @original_value = get_const_defined_on(@context, @const_name)

          @constants_to_transfer = verify_constants_to_transfer!

          @context.__send__(:remove_const, @const_name)
          @context.const_set(@const_name, @mutated_value)

          transfer_nested_constants
        end

        def to_constant
          constant = super
          constant.stubbed = true
          constant.previously_defined = true

          constant
        end

        def reset
          @constants_to_transfer.each do |constant|
            @mutated_value.__send__(:remove_const, constant)
          end

          @context.__send__(:remove_const, @const_name)
          @context.const_set(@const_name, @original_value)
        end

        def transfer_nested_constants
          @constants_to_transfer.each do |constant|
            @mutated_value.const_set(constant, get_const_defined_on(original_value, constant))
          end
        end

        def verify_constants_to_transfer!
          return [] unless should_transfer_nested_constants?

          { @original_value => "the original value", @mutated_value => "the stubbed value" }.each do |value, description|
            next if value.respond_to?(:constants)

            raise ArgumentError,
                  "Cannot transfer nested constants for #{@full_constant_name} " \
                  "since #{description} is not a class or module and only classes " \
                  "and modules support nested constants."
          end

          if Array === @transfer_nested_constants
            undefined_constants = @transfer_nested_constants - constants_defined_on(@original_value)

            if undefined_constants.any?
              available_constants = constants_defined_on(@original_value) - @transfer_nested_constants
              raise ArgumentError,
                    "Cannot transfer nested constant(s) #{undefined_constants.join(' and ')} " \
                    "for #{@full_constant_name} since they are not defined. Did you mean " \
                    "#{available_constants.join(' or ')}?"
            end

            @transfer_nested_constants
          else
            constants_defined_on(@original_value)
          end
        end

        def should_transfer_nested_constants?
          return true if @transfer_nested_constants
          return false unless ::Rex::Mocks.configuration.transfer_nested_constants?

          @original_value.respond_to?(:constants) && @mutated_value.respond_to?(:constants)
        end
      end

      class UndefinedConstantSetter < BaseMutator
        def mutate
          @parent = @context_parts.inject(Object) do |klass, name|
            if const_defined_on?(klass, name)
              get_const_defined_on(klass, name)
            else
              ConstantMutator.stub(name_for(klass, name), Module.new)
            end
          end

          @parent.const_set(@const_name, @mutated_value)
        end

        def to_constant
          constant = super
          constant.stubbed = true
          constant.previously_defined = false

          constant
        end

        def reset
          @parent.__send__(:remove_const, @const_name)
        end

        private

        def name_for(parent, name)
          root = parent == Object ? "" : parent.name
          root + "::" + name
        end
      end

      def self.mutate(mutator)
        ::Rex::Mocks.space.register_constant_mutator(mutator)
        mutator.mutate
      end

      def self.raise_on_invalid_const
        lambda do |const_name, failed_name|
          raise "Cannot stub constant #{failed_name} on #{const_name} " \
                "since #{const_name} is not a module."
        end
      end
    end
  end
end
