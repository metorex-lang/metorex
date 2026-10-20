module Rex
  module Mocks
    # Names a class or module a verifying double stands for, written as the
    # constant itself or as its name.
    class ObjectReference
      def self.for(object_module_or_name, allow_direct_object_refs = false)
        case object_module_or_name
        when Module then DirectModuleReference.new(object_module_or_name)
        when String then NamedObjectReference.new(object_module_or_name)
        else
          if allow_direct_object_refs
            DirectObjectReference.new(object_module_or_name)
          else
            raise ArgumentError, "Module or String expected, got #{object_module_or_name.inspect}"
          end
        end
      end
    end

    class DirectObjectReference
      def initialize(object)
        @object = object
      end

      def description
        @object.inspect
      end

      def const_to_replace
        @object.name
      end

      def defined?
        true
      end

      def target
        @object
      end

      def when_loaded
        yield @object
      end
    end

    class DirectModuleReference < DirectObjectReference
      def description
        @object.name || @object.inspect
      end
    end

    class NamedObjectReference
      def initialize(const_name)
        @const_name = const_name
      end

      def defined?
        !!object
      end

      def const_to_replace
        @const_name
      end
      alias description const_to_replace

      def target
        object
      end

      def when_loaded
        yield object if object
      end

      private

      def object
        return @object if defined?(@object) && @object

        @object = @const_name.split("::").reject(&:empty?).inject(Object) { |scope, name| scope.const_get(name) }
      rescue NameError
        @object = nil
      end
    end

    # Checks a call's arguments against a method's signature, with RSpec's
    # messages.
    class MethodSignatureVerifier
      attr_reader :non_kw_args, :kw_args

      def initialize(signature, args = [])
        @signature = signature
        @non_kw_args, @kw_args = split_args(*args)
      end

      def valid?
        missing_kw_args.empty? && invalid_kw_args.empty? && valid_non_kw_args?
      end

      def error_message
        if missing_kw_args.any?
          format("Missing required keyword arguments: %s", missing_kw_args.join(", "))
        elsif invalid_kw_args.any?
          format("Invalid keyword arguments provided: %s", invalid_kw_args.join(", "))
        elsif !valid_non_kw_args?
          format("Wrong number of arguments. Expected %s, got %s.", @signature.non_kw_args_arity_description, non_kw_args.length)
        end
      end

      private

      def valid_non_kw_args?
        @signature.valid_non_kw_args?(non_kw_args.length)
      end

      def missing_kw_args
        @signature.required_kw_args - @kw_args
      end

      def invalid_kw_args
        @signature.invalid_kw_args_from(@kw_args)
      end

      def split_args(*args)
        kw_args = if @signature.has_kw_args_in?(args) && Hash === args.last
                    last = args.pop
                    last.keys
                  else
                    []
                  end
        [args, kw_args]
      end
    end

    # A double that knows the class or object it stands for, and refuses a
    # message that the real one would not answer.
    module VerifyingDouble
      def respond_to?(message, include_private = false)
        return super unless null_object?

        method_ref = __mock_proxy.method_reference[message]
        case method_ref.visibility
        when :public then true
        when :private then include_private
        when :protected then include_private || RUBY_VERSION.to_f < 2.0
        else !method_ref.unimplemented?
        end
      end

      def method_missing(message, *args, &block)
        if null_object?
          __mock_proxy.ensure_publicly_implemented(message, self)
        else
          __mock_proxy.ensure_implemented(message)
        end
        super
      end
      ruby2_keywords(:method_missing)

      def send(name, *args, &block)
        __mock_proxy.ensure_publicly_implemented(name, self) unless null_object?
        __send__(name, *args, &block)
      end
      ruby2_keywords(:send)

      def initialize(doubled_module, *args)
        @doubled_module = doubled_module
        possible_name = args.first
        name = if String === possible_name || Symbol === possible_name
                 args.shift
               else
                 @description
               end
        super(name, *args)
        @__sending_message = nil
      end

      private

      def __doubled_reference
        @doubled_module
      end
    end

    class InstanceVerifyingDouble < TestDouble
      include VerifyingDouble

      def __build_proxy
        VerifyingProxy.new(self, ErrorGenerator.new(self), @doubled_module, InstanceMethodReference)
      end
    end

    class ObjectVerifyingDouble < TestDouble
      include VerifyingDouble

      def __build_proxy
        VerifyingProxy.new(self, ErrorGenerator.new(self), @doubled_module, ObjectMethodReference)
      end
    end

    class ClassVerifyingDouble < TestDouble
      include VerifyingDouble

      def __build_proxy
        VerifyingProxy.new(self, ErrorGenerator.new(self), @doubled_module, ClassMethodReference)
      end
    end

    # A method of the doubled class or object, with its visibility and its
    # signature when the real one is there to read.
    class MethodReference
      def self.for(object_reference, method_name)
        new(object_reference, method_name)
      end

      def initialize(object_reference, method_name)
        @object_reference = object_reference
        @method_name = method_name
      end

      def implemented?
        @object_reference.when_loaded { |module_or_object| return method_implemented?(module_or_object) }
        false
      end

      def unimplemented?
        @object_reference.when_loaded { |_module_or_object| return !implemented? }
        false
      end

      def defined?
        @object_reference.when_loaded { |module_or_object| return method_defined?(module_or_object) }
        false
      end

      def with_signature
        return unless (original = original_method)

        yield Rex::Support::MethodSignature.new(original)
      end

      def visibility
        @object_reference.when_loaded { |module_or_object| return visibility_from(module_or_object) }
        :public
      end

      private

      def original_method
        @object_reference.when_loaded { |module_or_object| return find_method(module_or_object) }
        nil
      end
    end

    class InstanceMethodReference < MethodReference
      private

      def method_implemented?(mod)
        Rex::Mocks::MethodReference.method_defined_at_any_visibility?(mod, @method_name)
      end

      def method_defined?(mod)
        method_implemented?(mod)
      end

      def find_method(mod)
        mod.instance_method(@method_name)
      rescue NameError
        nil
      end

      def visibility_from(mod)
        if mod.public_method_defined?(@method_name)
          :public
        elsif mod.private_method_defined?(@method_name)
          :private
        elsif mod.protected_method_defined?(@method_name)
          :protected
        end
      end
    end

    class ObjectMethodReference < MethodReference
      def self.for(object_reference, method_name)
        if ClassNewMethodReference.applies_to?(method_name) { object_reference.when_loaded { |object| object } }
          ClassNewMethodReference.new(object_reference, method_name)
        else
          super
        end
      end

      private

      def method_implemented?(object)
        object.respond_to?(@method_name, true)
      end

      def method_defined?(object)
        (class << object; self; end).method_defined?(@method_name)
      end

      def find_method(object)
        object.method(@method_name)
      rescue NameError
        nil
      end

      def visibility_from(object)
        if object.public_methods.include?(@method_name)
          :public
        elsif object.private_methods.include?(@method_name)
          :private
        elsif object.protected_methods.include?(@method_name)
          :protected
        end
      end
    end

    class ClassMethodReference < ObjectMethodReference
    end

    class ClassNewMethodReference < ObjectMethodReference
      def self.applies_to?(method_name)
        return false unless method_name == :new

        klass = yield
        return false unless klass.respond_to?(:new, true)

        klass.method(:new).owner == Class
      end

      def with_signature
        @object_reference.when_loaded do |klass|
          yield Rex::Support::MethodSignature.new(klass.instance_method(:initialize))
        end
      end
    end

    class MethodReference
      def self.method_defined_at_any_visibility?(mod, method_name)
        mod.method_defined?(method_name) || mod.private_method_defined?(method_name)
      end
    end

    # The proxy of a verifying double: each message is checked against the
    # doubled method when it is stubbed and when it arrives.
    class VerifyingProxy < TestDoubleProxy
      def initialize(object, error_generator, doubled_module, method_reference_class)
        super(object, error_generator)
        @doubled_module = doubled_module
        @method_reference_class = method_reference_class
        @method_reference = Hash.new { |hash, name| hash[name] = method_reference_class.for(@doubled_module, name) }
        @method_doubles = Hash.new do |hash, name|
          hash[name] = VerifyingMethodDouble.new(@object, name, self, @method_reference[name])
        end
      end

      def method_reference
        @method_reference
      end

      def visibility_for(method_name)
        @method_reference[method_name].visibility.to_s
      end

      def ensure_implemented(method_name)
        return unless @method_reference[method_name].unimplemented?

        @error_generator.raise_unimplemented_error(@doubled_module, method_name, @object)
      end

      def ensure_publicly_implemented(method_name, object)
        ensure_implemented(method_name)
        visibility = @method_reference[method_name].visibility
        return if visibility == :public

        @error_generator.raise_non_public_error(method_name, visibility)
      end

      def add_stub(method_name, opts = {}, &implementation)
        ensure_implemented(method_name)
        super
      end

      def add_simple_stub(method_name, *args)
        ensure_implemented(method_name)
        super
      end

      def add_message_expectation(method_name, opts = {}, &block)
        ensure_implemented(method_name)
        super
      end
    end

    # Validates the arguments of each call against the doubled method.
    class VerifyingMessageExpectation < MessageExpectation
      attr_accessor :method_reference

      def initialize(*args, &block)
        super
      end

      def with(*args, &block)
        super(*args, &block).tap do
          validate_expected_arguments! do |signature|
            example_call_site_args = [:an_arg] * signature.min_non_kw_args
            example_call_site_args << :kw_args_hash if signature.required_kw_args.any?
            @argument_list_matcher.resolve_expected_args_based_on(example_call_site_args)
          end
        end
      end
      ruby2_keywords(:with)

      private

      def validate_expected_arguments!
        return if method_reference.nil?

        method_reference.with_signature do |signature|
          args = yield signature
          verifier = MethodSignatureVerifier.new(signature, args)
          unless verifier.valid?
            raise ArgumentError, verifier.error_message
          end
        end
      end
    end

    class VerifyingMethodDouble < MethodDouble
      def initialize(object, method_name, proxy, method_reference)
        super(object, method_name, proxy)
        @method_reference = method_reference
      end

      def message_expectation_class
        VerifyingMessageExpectation
      end

      def add_expectation(*args, &block)
        super.tap { |expectation| expectation.method_reference = @method_reference }
      end

      def add_stub(*args, &block)
        super.tap { |stub| stub.method_reference = @method_reference }
      end

      def proxy_method_invoked(obj, *args, &block)
        validate_arguments!(args)
        super
      end
      ruby2_keywords(:proxy_method_invoked)

      def validate_arguments!(actual_args)
        @method_reference.with_signature do |signature|
          verifier = MethodSignatureVerifier.new(signature, actual_args)
          raise ArgumentError, verifier.error_message unless verifier.valid?
        end
      end
    end

    # The method double of a partial double that verifies: the arguments of
    # each call are checked against the object's own method.
    class VerifyingExistingMethodDouble < VerifyingMethodDouble
      def self.for(object, method_name, proxy)
        reference = DirectObjectReference.new(object)
        new(object, method_name, proxy, ObjectMethodReference.for(reference, method_name))
      end

      # The object's own method, read before the stub replaces it, is the
      # signature each call is checked against.
      def initialize(object, method_name, proxy, method_reference)
        super
        @original_signature = begin
          Rex::Support::MethodSignature.new(::Kernel.instance_method(:method).bind(object).call(method_name))
        rescue NameError
          nil
        end
      end

      def validate_arguments!(actual_args)
        return unless @original_signature

        verifier = MethodSignatureVerifier.new(@original_signature, actual_args)
        raise ArgumentError, verifier.error_message unless verifier.valid?
      end

      def unimplemented?
        false
      end
    end
  end

end
