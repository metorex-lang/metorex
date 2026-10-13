module Rex
  module Core
    # A block of examples or helpers written once and included by name.
    class SharedExampleGroup
      attr_reader :description, :definition, :metadata

      def initialize(description, metadata, definition)
        @description = description
        @metadata = metadata
        @definition = definition
      end

      # Evaluates the shared block in `host`, with the arguments the
      # inclusion passed, then the customization block written with it.
      def include_in(host, inclusion_line, args, customization_block)
        host.update_inherited_metadata(@metadata) unless @metadata.empty?
        SharedGroupInclusion.with_frame(@description, inclusion_line) do
          host.class_exec(*args, &@definition)
          host.class_exec(&customization_block) if customization_block
        end
      end
    end

    # Where each shared group being evaluated was included from, so an
    # example defined inside one records it.
    module SharedGroupInclusion
      Frame = Struct.new(:shared_group_name, :inclusion_location) do
        def description
          "Shared Example Group: #{shared_group_name.inspect} called from #{formatted_inclusion_location}"
        end

        def formatted_inclusion_location
          location = inclusion_location.to_s.sub(/:in .*\z/, "")
          Rex::Core.relative_path(location)
        end
      end

      def self.stack
        Thread.current[:rex_shared_group_inclusions] ||= []
      end

      def self.current_backtrace
        stack.reverse
      end

      def self.with_frame(name, location)
        stack.push(Frame.new(name, location))
        yield
      ensure
        stack.pop
      end
    end

    # The shared groups defined so far, by the group they were defined in,
    # or at the top level.
    class SharedExampleGroupRegistry
      TOP_LEVEL = :top_level

      def initialize
        @groups = Hash.new { |hash, context| hash[context] = {} }
      end

      def add(context, name, *metadata_args, &block)
        unless block
          raise ArgumentError, "Shared example group '#{name}' requires a block"
        end

        if valid_name?(name)
          metadata = Metadata.build_hash_from(metadata_args)
          shared = SharedExampleGroup.new(name, metadata_for_hosts(metadata), block)
          warn_if_key_taken(context, name)
          @groups[context][name] = shared
          register_automatic_inclusion(metadata, shared) unless metadata.empty?
        else
          metadata = Metadata.build_hash_from([name, *metadata_args])
          shared = SharedExampleGroup.new("anonymous shared context", {}, block)
          register_automatic_inclusion(metadata, shared)
        end
      end

      def find(lookup_contexts, name)
        lookup_contexts.each do |context|
          found = @groups[context][name]
          return found if found
        end
        @groups[TOP_LEVEL][name]
      end

      private

      def valid_name?(name)
        name.is_a?(String) || name.is_a?(Symbol) && !name.to_s.empty? || name.is_a?(Module)
      end

      def metadata_for_hosts(metadata)
        Rex.configuration.shared_context_metadata_behavior == :apply_to_host_groups ? metadata : {}
      end

      def register_automatic_inclusion(metadata, shared)
        return unless Rex.configuration.shared_context_metadata_behavior == :trigger_inclusion || !valid_name?(shared.description)

        Rex.configuration.automatic_shared_inclusions << [shared, metadata]
      end

      def warn_if_key_taken(context, name)
        existing = @groups[context][name]
        return unless existing

        Rex.configuration.error_stream.puts "WARNING: Shared example group '#{name}' has been previously defined at:\n" \
                                            "  #{existing.definition.source_location&.join(':')}\n" \
                                            "...and you are now defining it at:\n" \
                                            "  #{caller_locations.find { |location| !Rex::Core.framework_location?(location) }&.then { |location| "#{location.path}:#{location.lineno}" }}\n" \
                                            "The new definition will overwrite the original one."
      end
    end

    # `shared_examples`, `shared_examples_for`, and `shared_context`, at the
    # top level and inside a group.
    module SharedExampleGroupDefinitions
      def shared_examples(name, *args, &block)
        context = is_a?(Class) && self <= ExampleGroup ? self : SharedExampleGroupRegistry::TOP_LEVEL
        Rex.world.shared_example_group_registry.add(context, name, *args, &block)
      end
      alias shared_context shared_examples
      alias shared_examples_for shared_examples
    end

    class ExampleGroup
      extend SharedExampleGroupDefinitions

      class << self
        def include_context(name, *args, &block)
          find_and_eval_shared("context", name, caller_line_for_inclusion, *args, &block)
        end

        def include_examples(name, *args, &block)
          find_and_eval_shared("examples", name, caller_line_for_inclusion, *args, &block)
        end

        def it_behaves_like(name, *args, &customization_block)
          nested_shared_group("behaves like", name, *args, &customization_block)
        end

        def it_should_behave_like(name, *args, &customization_block)
          nested_shared_group("it should behave like", name, *args, &customization_block)
        end

        # `config.alias_it_behaves_like_to`: another name for
        # `it_behaves_like`, whose group is described with `report_label`.
        def define_nested_shared_group_method(new_name, report_label)
          define_singleton_method(new_name) do |name, *args, &customization_block|
            nested_shared_group(report_label, name, *args, &customization_block)
          end
        end

        def update_inherited_metadata(updates)
          metadata.update(updates) { |_key, existing, _new| existing }
          children.each { |child| child.update_inherited_metadata(updates) }
          examples.each { |example| example.metadata.update(updates) { |_key, existing, _new| existing } }
        end

        private

        def nested_shared_group(label, name, *args, &customization_block)
          inclusion_line = caller_line_for_inclusion
          location = Metadata.location_from(caller_locations(1, 8))
          group = subclass("#{label} #{name}", location: location, extra_metadata: { shared_group_name: name })
          group.__send__(:find_and_eval_shared, "examples", name, inclusion_line, *args, &customization_block)
          group
        end

        def caller_line_for_inclusion
          frame = caller_locations.find { |location| !Rex::Core.framework_location?(location) }
          frame ? "#{frame.path}:#{frame.lineno}" : nil
        end

        def find_and_eval_shared(label, name, inclusion_location, *args, &customization_block)
          shared = Rex.world.shared_example_group_registry.find(parent_groups, name)
          raise ArgumentError, "Could not find shared #{label} #{name.inspect}" unless shared

          shared.include_in(self, inclusion_location, args, customization_block)
        end
      end
    end

    class World
      def shared_example_group_registry
        @shared_example_group_registry ||= SharedExampleGroupRegistry.new
      end
    end

    class Configuration
      def automatic_shared_inclusions
        @automatic_shared_inclusions ||= []
      end

      # `config.include_context "name", :tag`: every group whose metadata
      # matches includes the shared context.
      def include_context(name, *metadata_args)
        (@named_context_inclusions ||= []) << [name, Metadata.build_hash_from(metadata_args)]
      end

      def named_context_inclusions
        @named_context_inclusions || []
      end
    end
  end

  extend Core::SharedExampleGroupDefinitions
end
