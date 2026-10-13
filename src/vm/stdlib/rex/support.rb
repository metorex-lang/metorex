require "rbconfig"

module Rex
  # What rspec-support gives the other RSpec libraries: where failures and
  # warnings go, and the exceptions a matcher must not rescue.
  module Support
    # `RSpec::Support.require_rspec_support "name"` and its siblings require
    # `rspec/<library>/<name>`, which Rex answers for the files it carries
    # and the load path answers for the rest, such as a gem's spec helpers.
    %i[support core expectations matchers mocks].each do |library|
      define_singleton_method("require_rspec_#{library}") do |name|
        require "rspec/#{library}/#{name}"
      end
    end

    module OS
      module_function

      def windows?
        !!(RbConfig::CONFIG["host_os"] =~ /cygwin|mswin|mingw|bccwin|wince|emx/)
      end

      def windows_file_path?
        ::File::ALT_SEPARATOR == "\\"
      end
    end

    module Ruby
      module_function

      def jruby?
        RUBY_PLATFORM == "java"
      end

      def jruby_9000?
        false
      end

      def rbx?
        RUBY_ENGINE == "rbx"
      end

      def non_mri?
        !mri?
      end

      def mri?
        RUBY_ENGINE == "ruby"
      end

      def truffleruby?
        RUBY_ENGINE == "truffleruby"
      end
    end

    module RubyFeatures
      module_function

      def fork_supported?
        Process.respond_to?(:fork)
      end

      def optional_and_splat_args_supported?
        Method.method_defined?(:parameters)
      end

      def caller_locations_supported?
        respond_to?(:caller_locations, true)
      end

      def supports_exception_cause?
        true
      end

      def supports_syntax_suggest?
        true
      end

      def kw_arg_separation?
        true
      end

      def supports_taint?
        false
      end

      def ripper_supported?
        true
      end

      def distincts_kw_args_from_positional_hash?
        true
      end

      def kw_args_supported?
        true
      end

      def required_kw_args_supported?
        true
      end

      def supports_rebinding_module_methods?
        true
      end

      def module_refinement_supported?
        Module.method_defined?(:refine) || Module.private_method_defined?(:refine)
      end

      def module_prepends_supported?
        Module.method_defined?(:prepend) || Module.private_method_defined?(:prepend)
      end
    end

    def self.thread_local_data
      Thread.current.thread_variable_get(:__rspec) || Thread.current.thread_variable_set(:__rspec, {})
    end

    DEFAULT_FAILURE_NOTIFIER = ->(failure, _options) { raise failure }

    def self.failure_notifier=(callable)
      thread_local_data[:failure_notifier] = callable
    end

    def self.failure_notifier
      thread_local_data[:failure_notifier] || DEFAULT_FAILURE_NOTIFIER
    end

    def self.notify_failure(failure, options = {})
      failure_notifier.call(failure, options)
    end

    def self.with_failure_notifier(callable)
      original_notifier = failure_notifier
      self.failure_notifier = callable
      yield
    ensure
      self.failure_notifier = original_notifier
    end

    class << self
      attr_writer :warning_notifier
    end

    DEFAULT_WARNING_NOTIFIER = ->(warning) { ::Kernel.warn warning }

    def self.warning_notifier
      @warning_notifier ||= DEFAULT_WARNING_NOTIFIER
    end

    module AllExceptionsExceptOnesWeMustNotRescue
      AVOID_RESCUING = [NoMemoryError, SignalException, Interrupt, SystemExit].freeze

      def self.===(exception)
        AVOID_RESCUING.none? { |avoided| avoided === exception }
      end
    end

    def self.matcher_definitions
      @matcher_definitions ||= []
    end

    # How rspec-mocks recognizes rspec-expectations' matchers without
    # depending on it.
    def self.register_matcher_definition(&block)
      matcher_definitions << block
    end

    def self.deregister_matcher_definition(&block)
      matcher_definitions.delete(block)
    end

    def self.is_a_matcher?(object)
      matcher_definitions.any? { |definition| definition.call(object) }
    end

    def self.rspec_description_for_object(object)
      if is_a_matcher?(object) && object.respond_to?(:description)
        object.description
      else
        object
      end
    end

    KERNEL_METHOD_METHOD = ::Kernel.instance_method(:method)

    def self.method_handle_for(object, method_name)
      KERNEL_METHOD_METHOD.bind(object).call(method_name)
    rescue NameError => original
      begin
        handle = object.method(method_name)
        raise original unless handle.is_a? Method

        handle
      rescue AllExceptionsExceptOnesWeMustNotRescue
        raise original
      end
    end

    def self.class_of(object)
      object.class
    rescue NoMethodError
      singleton_class = class << object; self; end
      singleton_class.ancestors.find { |ancestor| !ancestor.equal?(singleton_class) }
    end
  end
end

module Rex
  module Support
    # Reading and checking constants by a name such as `"A::B::C"`.
    module RecursiveConstMethods
      def const_defined_on?(mod, const_name)
        mod.const_defined?(const_name, false)
      end

      def get_const_defined_on(mod, const_name)
        mod.const_get(const_name, false)
      end

      def constants_defined_on(mod)
        mod.constants(false)
      end

      def recursive_const_get(const_name)
        normalize_const_name(const_name).split("::").inject(Object) do |mod, name|
          get_const_defined_on(mod, name)
        end
      end

      def recursive_const_defined?(const_name)
        parts = normalize_const_name(const_name).split("::")
        parts.inject([Object, ""]) do |(mod, full_name), name|
          yield(full_name, name) if block_given? && !(Module === mod)
          return false unless const_defined_on?(mod, name)

          [get_const_defined_on(mod, name), [mod.name, name].join("::")]
        end
      end

      def normalize_const_name(const_name)
        const_name.sub(/\A::/, "")
      end
    end
  end
end

module Rex
  module Support
    # The warnings and deprecations the RSpec libraries print.
    module Warnings
      def deprecate(deprecated, options = {})
        warn_with "DEPRECATION: #{deprecated} is deprecated.", options
      end

      def warn_deprecation(message, options = {})
        warn_with "DEPRECATION: \n #{message}", options
      end

      def warning(text, options = {})
        warn_with "WARNING: #{text}.", options
      end

      # Hands a message naming where it was called from to the warning
      # notifier, which prints it, or raises it when `raise_on_warning` is on.
      def warn_with(message, options = {})
        call_site = options.fetch(:call_site) { CallerFilter.first_non_rspec_line }
        message += " Use #{options[:replacement]} instead." if options[:replacement]
        message += " Called from #{call_site}." if call_site
        Support.warning_notifier.call message
      end
    end
  end
end
