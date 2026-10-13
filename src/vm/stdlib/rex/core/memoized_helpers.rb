module Rex
  module Core
    # `let`, `subject`, and the one-liner syntax that reads the subject.
    module MemoizedHelpers
      def subject
        __memoized.fetch(:subject) do
          __memoized[:subject] = begin
            described = described_class || self.class.metadata[:description_args].first
            described.is_a?(Class) ? described.new : described
          end
        end
      end

      def should(matcher = nil, message = nil)
        Expectations::ExpectationTarget.new(subject).to(matcher, message)
      end

      def should_not(matcher = nil, message = nil)
        Expectations::ExpectationTarget.new(subject).not_to(matcher, message)
      end

      def is_expected
        expect(subject)
      end

      def __memoized
        @__memoized ||= {}
      end

      # The module a group's `let` definitions go in. The group includes it,
      # so a `let` redefined in a nested group can call `super`.
      def self.module_for(group)
        if group.const_defined?(:LetDefinitions, false)
          group.const_get(:LetDefinitions, false)
        else
          definitions = Module.new
          group.const_set(:LetDefinitions, definitions)
          group.__send__(:include, definitions)
          definitions
        end
      end

      module ClassMethods
        def let(name, &block)
          raise "#let or #subject called without a block" if block.nil?
          raise "#let or #subject called with a reserved name #initialize" if name.to_sym == :initialize

          name = name.to_sym
          MemoizedHelpers.module_for(self).__send__(:define_method, name, &block)
          define_method(name) do
            __memoized.fetch(name) { __memoized[name] = super(&nil) }
          end
          name
        end

        def let!(name, &block)
          let(name, &block)
          before { __send__(name) }
        end

        def subject(name = nil, &block)
          if name
            let(name, &block)
            alias_method :subject, name
          else
            let(:subject, &block)
          end
        end

        def subject!(name = nil, &block)
          subject(name, &block)
          before { subject }
        end
      end
    end
  end
end
