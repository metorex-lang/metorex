# frozen_string_literal: true

require "rex/support/method_signature_verifier"

module Rex
  module Support
    module WithKeywordsWhenNeeded

      module_function

      if Rex::Support::RubyFeatures.kw_args_supported?

        def class_exec(klass, *args, &block)
          if MethodSignature.new(block).has_kw_args_in?(args)
            binding.eval(<<-CODE, __FILE__, __LINE__)
            kwargs = args.pop
            klass.class_exec(*args, **kwargs, &block)
            CODE
          else
            klass.class_exec(*args, &block)
          end
        end
        ruby2_keywords :class_exec if respond_to?(:ruby2_keywords, true)
      else
        def class_exec(klass, *args, &block)
          klass.class_exec(*args, &block)
        end
      end
    end
  end
end
