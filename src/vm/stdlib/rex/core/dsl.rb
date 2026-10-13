module Rex
  module Core
    module DSL
      def self.example_group_aliases
        @example_group_aliases ||= []
      end

      def self.exposed_globally?
        @exposed_globally ||= false
      end

      def self.expose_example_group_alias(name)
        return if example_group_aliases.include?(name)

        example_group_aliases << name

        (class << Rex; self; end).__send__(:define_method, name) do |*args, &example_group_block|
          group = Rex::Core::ExampleGroup.__send__(name, *args, &example_group_block)
          Rex.world.record(group)
          group
        end

        expose_example_group_alias_globally(name) if exposed_globally?
      end

      class << self
        attr_accessor :top_level
      end

      def self.expose_globally!
        return if exposed_globally?

        example_group_aliases.each do |method_name|
          expose_example_group_alias_globally(method_name)
        end

        @exposed_globally = true
      end

      def self.remove_globally!
        return unless exposed_globally?

        example_group_aliases.each do |method_name|
          change_global_dsl { undef_method method_name }
        end

        @exposed_globally = false
      end

      def self.expose_example_group_alias_globally(method_name)
        change_global_dsl do
          remove_method(method_name) if method_defined?(method_name)
          define_method(method_name) { |*a, &b| ::Rex.__send__(method_name, *a, &b) }
        end
      end

      def self.change_global_dsl(&changes)
        (class << top_level; self; end).class_exec(&changes)
        Module.class_exec(&changes)
      end
    end
  end
end

::Rex::Core::DSL.top_level = self
