module Rex
  module Core
    # `Rex.describe` and its aliases, and the same methods on `main` and on
    # every Module while the DSL is exposed globally.
    module DSL
      GROUP_METHODS = %i[example_group describe context xdescribe xcontext fdescribe fcontext].freeze
      SHARED_METHODS = %i[shared_examples shared_examples_for shared_context].freeze

      GROUP_METHODS.each do |name|
        define_method(name) do |*arguments, &block|
          location = Metadata.location_from(caller_locations(1, 8))
          extra = case name
                  when :xdescribe then { skip: "Temporarily skipped with xdescribe" }
                  when :xcontext then { skip: "Temporarily skipped with xcontext" }
                  when :fdescribe, :fcontext then { focus: true }
                  else {}
                  end
          ExampleGroup.subclass(*arguments, location: location, extra_metadata: extra, &block)
        end
      end

      def self.expose_globally!
        return if @exposed

        @exposed = true
        aliases.each { |name| expose_name_globally(name) }
        main = TOPLEVEL_BINDING.receiver
        GROUP_METHODS.each do |name|
          main.define_singleton_method(name) do |*arguments, &block|
            Rex.__send__(name, *arguments, &block)
          end
          Module.__send__(:define_method, name) do |*arguments, &block|
            Rex.__send__(name, *arguments, &block)
          end
        end
        SHARED_METHODS.each do |name|
          main.define_singleton_method(name) do |*arguments, &block|
            Rex.__send__(name, *arguments, &block)
          end
          Module.__send__(:define_method, name) do |*arguments, &block|
            Rex.__send__(name, *arguments, &block)
          end
        end
      end

      # A name `config.alias_example_group_to` adds, on Rex and, while the
      # DSL is exposed, on `main` and every Module.
      def self.expose_example_group_alias(name)
        aliases << name
        define_method(name) do |*arguments, &block|
          Rex::Core::ExampleGroup.__send__(name, *arguments, &block)
        end
        Rex.singleton_class.__send__(:define_method, name, instance_method(name))
        expose_name_globally(name) if @exposed
      end

      def self.aliases
        @aliases ||= []
      end

      def self.expose_name_globally(name)
        main = TOPLEVEL_BINDING.receiver
        main.define_singleton_method(name) { |*arguments, &block| Rex.__send__(name, *arguments, &block) }
        Module.__send__(:define_method, name) { |*arguments, &block| Rex.__send__(name, *arguments, &block) }
      end

      def self.remove_globally!
        return unless @exposed

        @exposed = false
        main = TOPLEVEL_BINDING.receiver
        (GROUP_METHODS + SHARED_METHODS + aliases).each do |name|
          main.singleton_class.__send__(:remove_method, name) if main.singleton_class.method_defined?(name, false)
          Module.__send__(:remove_method, name) if Module.method_defined?(name, false)
        end
      end
    end
  end

  extend Core::DSL
end
