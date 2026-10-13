module Rex
  module Mocks
    # What `config.mock_with(:rspec) { |c| }` yields, and
    # `RSpec::Mocks.configuration`.
    class Configuration
      def initialize
        @allow_message_expectations_on_nil = nil
        @yield_receiver_to_any_instance_implementation_blocks = true
        @verify_doubled_constant_names = false
        @transfer_nested_constants = false
        @verify_partial_doubles = false
        @temporarily_suppress_partial_double_verification = false
        @color = false
      end

      attr_accessor :allow_message_expectations_on_nil

      def yield_receiver_to_any_instance_implementation_blocks?
        @yield_receiver_to_any_instance_implementation_blocks
      end

      attr_writer :yield_receiver_to_any_instance_implementation_blocks

      def add_stub_and_should_receive_to(*modules)
        modules.each do |mod|
          Syntax.enable_should(mod)
        end
      end

      def syntax=(*values)
        syntaxes = values.flatten
        if syntaxes.include?(:expect)
          Syntax.enable_expect
        else
          Syntax.disable_expect
        end

        if syntaxes.include?(:should)
          Syntax.enable_should
        else
          Syntax.disable_should
        end
      end

      def syntax
        syntaxes = []
        syntaxes << :should if Syntax.should_enabled?
        syntaxes << :expect if Syntax.expect_enabled?
        syntaxes
      end

      def verify_doubled_constant_names?
        !!@verify_doubled_constant_names
      end

      attr_writer :verify_doubled_constant_names

      def before_verifying_doubles(&block)
        verifying_double_callbacks << block
      end
      alias when_declaring_verifying_double before_verifying_doubles

      def verifying_double_callbacks
        @verifying_double_callbacks ||= []
      end

      def transfer_nested_constants?
        !!@transfer_nested_constants
      end

      attr_writer :transfer_nested_constants

      def verify_partial_doubles=(value)
        @verify_partial_doubles = !!value
      end

      def verify_partial_doubles?
        @verify_partial_doubles
      end

      attr_accessor :temporarily_suppress_partial_double_verification

      def color?
        ::Rex.configuration.color_enabled?
      end

      def patch_marshal_to_support_partial_doubles=(value)
        if value
          MarshalExtension.patch!
        else
          MarshalExtension.unpatch!
        end
      end

      def reset_syntaxes_to_default
        self.syntax = [:should, :expect]
        Syntax.warn_about_should!
      end
    end

    # `Marshal.dump` of a partial double dumps a copy, which carries none of
    # the singleton methods its stubs added.
    class MarshalExtension
      def self.patch!
        return if Marshal.respond_to?(:dump_with_rspec_mocks)

        Marshal.instance_eval do
          class << self
            def dump_with_rspec_mocks(object, *rest)
              if !::Rex::Mocks.space.registered?(object) || NilClass === object
                dump_without_rspec_mocks(object, *rest)
              else
                dump_without_rspec_mocks(object.dup, *rest)
              end
            end

            alias_method :dump_without_rspec_mocks, :dump
            undef_method :dump
            alias_method :dump, :dump_with_rspec_mocks
          end
        end
      end

      def self.unpatch!
        return unless Marshal.respond_to?(:dump_with_rspec_mocks)

        Marshal.instance_eval do
          class << self
            undef_method :dump_with_rspec_mocks
            undef_method :dump
            alias_method :dump, :dump_without_rspec_mocks
            undef_method :dump_without_rspec_mocks
          end
        end
      end
    end

    # Where `should_receive`, `stub`, `allow`, and the rest are defined, and
    # whether each is.
    module Syntax
      def self.warn_about_should!
        @warn_about_should = true
      end

      def self.warn_unless_should_configured(method_name, replacement = "the new `:expect` syntax or explicitly enable `:should`")
        return unless @warn_about_should

        ::Rex.deprecate(
          "Using `#{method_name}` from rspec-mocks' old `:should` syntax without explicitly enabling the syntax",
          replacement: replacement
        )
        @warn_about_should = false
      end

      def self.enable_should(syntax_host = default_should_syntax_host)
        @warn_about_should = false if syntax_host == default_should_syntax_host
        return if should_enabled?(syntax_host)

        syntax_host.class_exec do
          def should_receive(message, opts = {}, &block)
            ::Rex::Mocks::Syntax.warn_unless_should_configured(__method__)
            ::Rex::Mocks.expect_message(self, message, opts, &block)
          end

          def should_not_receive(message, &block)
            ::Rex::Mocks::Syntax.warn_unless_should_configured(__method__)
            ::Rex::Mocks.expect_message(self, message, {}, &block).never
          end

          def stub(message_or_hash, opts = {}, &block)
            ::Rex::Mocks::Syntax.warn_unless_should_configured(__method__)
            if ::Hash === message_or_hash
              message_or_hash.each { |message, value| stub(message).and_return value }
            else
              ::Rex::Mocks.allow_message(self, message_or_hash, opts, &block)
            end
          end

          def unstub(message)
            ::Rex::Mocks::Syntax.warn_unless_should_configured(
              __method__,
              "`allow(...).to receive(...).and_call_original` or explicitly enable `:should`"
            )
            ::Rex::Mocks.space.proxy_for(self).remove_stub(message)
          end

          def stub_chain(*chain, &block)
            ::Rex::Mocks::Syntax.warn_unless_should_configured(__method__)
            ::Rex::Mocks::Matchers::ReceiveMessageChain.new(chain, &block).setup_allowance(self)
          end

          def as_null_object
            ::Rex::Mocks::Syntax.warn_unless_should_configured(__method__)
            @_null_object = true
            ::Rex::Mocks.space.proxy_for(self).as_null_object
          end

          def null_object?
            ::Rex::Mocks::Syntax.warn_unless_should_configured(__method__)
            defined?(@_null_object)
          end

          def received_message?(message, *args, &block)
            ::Rex::Mocks::Syntax.warn_unless_should_configured(__method__)
            ::Rex::Mocks.space.proxy_for(self).received_message?(message, *args, &block)
          end

          unless Class.respond_to? :any_instance
            Class.class_exec do
              def any_instance
                ::Rex::Mocks::Syntax.warn_unless_should_configured(__method__)
                ::Rex::Mocks.space.any_instance_proxy_for(self)
              end
            end
          end
        end
      end

      def self.disable_should(syntax_host = default_should_syntax_host)
        return unless should_enabled?(syntax_host)

        syntax_host.class_exec do
          undef should_receive
          undef should_not_receive
          undef stub
          undef unstub
          undef stub_chain
          undef as_null_object
          undef null_object?
          undef received_message?
        end

        Class.class_exec do
          undef any_instance
        end
      end

      def self.enable_expect(syntax_host = ::Rex::Mocks::ExampleMethods)
        return if expect_enabled?(syntax_host)

        syntax_host.class_exec do
          def receive(method_name, &block)
            ::Rex::Mocks::Matchers::Receive.new(method_name, block)
          end

          def receive_messages(message_return_value_hash, &_block)
            ::Rex::Mocks::Matchers::ReceiveMessages.new(message_return_value_hash)
          end

          def receive_message_chain(*messages, &block)
            ::Rex::Mocks::Matchers::ReceiveMessageChain.new(messages, &block)
          end

          def allow(target)
            ::Rex::Mocks::AllowanceTarget.new(target)
          end

          def expect_any_instance_of(klass)
            ::Rex::Mocks::AnyInstanceExpectationTarget.new(klass)
          end

          def allow_any_instance_of(klass)
            ::Rex::Mocks::AnyInstanceAllowanceTarget.new(klass)
          end
        end
      end

      def self.disable_expect(syntax_host = ::Rex::Mocks::ExampleMethods)
        return unless expect_enabled?(syntax_host)

        syntax_host.class_exec do
          undef receive
          undef receive_messages
          undef receive_message_chain
          undef allow
          undef expect_any_instance_of
          undef allow_any_instance_of
        end
      end

      def self.should_enabled?(syntax_host = default_should_syntax_host)
        syntax_host.method_defined?(:should_receive)
      end

      def self.expect_enabled?(syntax_host = ::Rex::Mocks::ExampleMethods)
        syntax_host.method_defined?(:allow)
      end

      def self.default_should_syntax_host
        ::BasicObject
      end
    end

    def self.configuration
      @configuration ||= Configuration.new
    end
  end
end
