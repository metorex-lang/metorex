require "rex/support/with_keywords_when_needed"

module Rex
  module Core
    class SharedExampleGroupModule < Module
      attr_reader :definition

      def initialize(description, definition, metadata)
        @description = description
        @definition  = definition
        @metadata    = metadata
      end

      def inspect
        "#<#{self.class.name} #{@description.inspect}>"
      end
      alias to_s inspect

      def included(klass)
        inclusion_line = klass.metadata[:location]
        include_in klass, inclusion_line, [], nil
      end

      def include_in(klass, inclusion_line, args, customization_block)
        klass.update_inherited_metadata(@metadata) unless @metadata.empty?

        SharedExampleGroupInclusionStackFrame.with_frame(@description, inclusion_line) do
          Rex::Support::WithKeywordsWhenNeeded.class_exec(klass, *args, &@definition)
          klass.class_exec(&customization_block) if customization_block
        end
      end
    end

    module SharedExampleGroup
      def shared_examples(name, *args, &block)
        top_level = self == ExampleGroup
        if top_level && Rex::Support.thread_local_data[:in_example_group]
          raise "Creating isolated shared examples from within a context is " \
                "not allowed. Remove `RSpec.` prefix or move this to a " \
                "top-level scope."
        end

        Rex.world.shared_example_group_registry.add(self, name, *args, &block)
      end
      alias shared_context      shared_examples
      alias shared_examples_for shared_examples

      module TopLevelDSL
        def self.definitions
          proc do
            def shared_examples(name, *args, &block)
              Rex.world.shared_example_group_registry.add(:main, name, *args, &block)
            end
            alias shared_context      shared_examples
            alias shared_examples_for shared_examples
          end
        end

        def self.exposed_globally?
          @exposed_globally ||= false
        end

        def self.expose_globally!
          return if exposed_globally?
          Core::DSL.change_global_dsl(&definitions)
          @exposed_globally = true
        end

        def self.remove_globally!
          return unless exposed_globally?

          Core::DSL.change_global_dsl do
            undef shared_examples
            undef shared_context
            undef shared_examples_for
          end

          @exposed_globally = false
        end
      end

      class Registry
        def add(context, name, *metadata_args, &block)
          unless block
            Rex.warning "Shared example group #{name} was defined without a "\
                          "block and will have no effect. Please define a "\
                          "block or remove the definition."
          end

          if Rex.configuration.shared_context_metadata_behavior == :trigger_inclusion
            return legacy_add(context, name, *metadata_args, &block)
          end

          unless valid_name?(name)
            raise ArgumentError, "Shared example group names can only be a string, " \
                                 "symbol or module but got: #{name.inspect}"
          end

          ensure_block_has_source_location(block) { CallerFilter.first_non_rspec_line }
          warn_if_key_taken context, name, block

          metadata = Metadata.build_hash_from(metadata_args)
          shared_module = SharedExampleGroupModule.new(name, block, metadata)
          shared_example_groups[context][name] = shared_module
        end

        def find(lookup_contexts, name)
          lookup_contexts.each do |context|
            found = shared_example_groups[context][name]
            return found if found
          end

          shared_example_groups[:main][name]
        end

      private

        def legacy_add(context, name, *metadata_args, &block)
          ensure_block_has_source_location(block) { CallerFilter.first_non_rspec_line }
          shared_module = SharedExampleGroupModule.new(name, block, {})

          if valid_name?(name)
            warn_if_key_taken context, name, block
            shared_example_groups[context][name] = shared_module
          else
            metadata_args.unshift name
          end

          return if metadata_args.empty?
          Rex.configuration.include shared_module, *metadata_args
        end

        def shared_example_groups
          @shared_example_groups ||= Hash.new { |hash, context| hash[context] = {} }
        end

        def valid_name?(candidate)
          case candidate
          when String, Symbol, Module then true
          else false
          end
        end

        def warn_if_key_taken(context, key, new_block)
          existing_module = shared_example_groups[context][key]
          return unless existing_module

          old_definition_location = formatted_location existing_module.definition
          new_definition_location = formatted_location new_block
          loaded_spec_files = Rex.configuration.loaded_spec_files

          if loaded_spec_files.include?(new_definition_location) && old_definition_location == new_definition_location
            Rex.warn_with <<-WARNING.gsub(/^ +\|/, ''), :call_site => nil
              |WARNING: Your shared example group, '#{key}', defined at:
              | #{old_definition_location}
              |was automatically loaded by RSpec because the file name
              |matches the configured autoloading pattern (#{Rex.configuration.pattern}),
              |and is also being required from somewhere else. To fix this
              |warning, either rename the file to not match the pattern, or
              |do not explicitly require the file.
            WARNING
          else
            Rex.warn_with <<-WARNING.gsub(/^ +\|/, ''), :call_site => nil
              |WARNING: Shared example group '#{key}' has been previously defined at:
              |  #{old_definition_location}
              |...and you are now defining it at:
              |  #{new_definition_location}
              |The new definition will overwrite the original one.
            WARNING
          end
        end

        if RUBY_VERSION.to_f >= 1.9
          def formatted_location(block)
            block.source_location.join(":")
          end
        else
          def formatted_location(block)
            block.source_location.join(":").gsub(/:in.*$/, '')
          end
        end

        if Proc.method_defined?(:source_location)
          def ensure_block_has_source_location(_block); end
        else
          def ensure_block_has_source_location(block)
            source_location = yield.split(':')
            block.extend(Module.new { define_method(:source_location) { source_location } })
          end
        end
      end
    end
  end

  instance_exec(&Core::SharedExampleGroup::TopLevelDSL.definitions)
end
