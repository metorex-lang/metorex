module Rex
  module Core
    # A group written with `describe`. Each group is a subclass of its parent
    # group, and each example runs in a new instance of its group.
    class ExampleGroup
      include MemoizedHelpers
      include Pending
      extend MemoizedHelpers::ClassMethods

      GROUP_ONLY_KEYS = [:parent_example_group, :example_group, :block, :scoped_id,
                         :description_args, :description, :full_description,
                         :file_path, :line_number, :location, :absolute_file_path].freeze

      # Every group has these, set when the group is defined.
      class << self
        def metadata
          @metadata ||= {}
        end

        def description
          description = metadata[:description]
          description = description.nil? || description.empty? ? to_s : description
          Rex.configuration.format_docstrings_block.call(description)
        end

        def described_class
          metadata[:described_class]
        end

        def hooks
          @hooks ||= Hooks::Collection.new
        end

        def examples
          @examples ||= []
        end

        def children
          @children ||= []
        end

        def declaration_line
          metadata[:line_number]
        end

        def file_path
          metadata[:file_path]
        end

        def location
          metadata[:location]
        end

        def id
          "#{metadata[:rerun_file_path]}[#{metadata[:scoped_id]}]"
        end

        def top_level?
          superclass == ExampleGroup
        end

        def top_level_description
          parent_groups.last.description
        end

        # This group and the groups it sits in, innermost first.
        def parent_groups
          groups = []
          group = self
          while group < ExampleGroup
            groups << group
            group = group.superclass
          end
          groups
        end

        def descendants
          [self] + children.flat_map(&:descendants)
        end

        # The `let` and `subject` names this group and the groups around it
        # declare, innermost first.
        def let_names
          parent_groups.flat_map do |group|
            next [] unless group.const_defined?(:LetDefinitions, false)

            group.const_get(:LetDefinitions, false).instance_methods(false).sort
          end.uniq
        end

        def inspect
          name || "#<Class:#{description}>"
        end

        def to_s
          name || super
        end

        def set_it_up(description_args, user_metadata, location, parent_metadata)
          file_path, line_number = location
          @metadata = parent_metadata ? parent_metadata.reject { |key, _| GROUP_ONLY_KEYS.include?(key) } : {}
          @metadata.update(user_metadata)
          @metadata[:description_args] = description_args
          @metadata[:description] = ExampleGroup.build_description_from(*description_args)
          @metadata[:parent_example_group] = parent_metadata
          @metadata[:described_class] = ExampleGroup.described_class_from(description_args, parent_metadata)
          @metadata[:file_path] = file_path
          @metadata[:line_number] = line_number
          @metadata[:location] = "#{file_path}:#{line_number}"
          @metadata[:absolute_file_path] = File.expand_path(file_path)
          # A group, and every example in it, reruns through the file its
          # outermost group was written in, which a shared group's support
          # file is not.
          @metadata[:rerun_file_path] = parent_metadata ? parent_metadata[:rerun_file_path] : file_path
          @metadata[:full_description] = ExampleGroup.full_description_for(@metadata)
          Rex.configuration.derived_metadata_blocks.dup.each do |filter, derive|
            derive.call(@metadata) if Metadata.filter_applies?(filter, @metadata)
          end
          Rex.configuration.inclusions.each do |mod, filter|
            include(mod) if Metadata.filter_applies?(filter, @metadata)
          end
          Rex.configuration.prependings.each do |mod, filter|
            prepend(mod) if Metadata.filter_applies?(filter, @metadata)
          end
          Rex.configuration.extensions.each do |mod, filter|
            extend(mod) if Metadata.filter_applies?(filter, @metadata)
          end
          include_matching_shared_contexts(parent_metadata)
        end

        # The shared contexts whose metadata this group matches, unless the
        # group it sits in matched them already and passed them down.
        def include_matching_shared_contexts(parent_metadata)
          matches = lambda do |filter|
            Metadata.filter_applies?(filter, @metadata) &&
              !(parent_metadata && Metadata.filter_applies?(filter, parent_metadata))
          end
          Rex.configuration.named_context_inclusions.each do |name, filter|
            next unless matches.call(filter)

            shared = Rex.world.shared_example_group_registry.find([], name)
            raise ArgumentError, "Could not find shared context #{name.inspect}" unless shared

            shared.include_in(self, nil, [], nil)
          end
          Rex.configuration.automatic_shared_inclusions.each do |shared, filter|
            shared.include_in(self, nil, [], nil) if matches.call(filter)
          end
        end

        # `describe` inside a group: a subclass of this group.
        def subclass(*arguments, location:, extra_metadata: {}, &block)
          description = arguments.shift
          rest, user_metadata = Metadata.split(arguments)
          description_args = description.nil? ? rest : [description, *rest]
          user_metadata = user_metadata.merge(extra_metadata)

          group = Class.new(self)
          group.set_it_up(description_args, user_metadata, location, self == ExampleGroup ? nil : metadata)
          if self == ExampleGroup
            Rex.world.register(group)
          else
            children << group
          end
          group.metadata[:scoped_id] = group.scoped_id_from_parent
          Rex::ExampleGroups.assign_const(group)
          group.module_exec(&block) if block
          if self == ExampleGroup
            Rex.configuration.on_example_group_definition_callbacks.each { |callback| callback.call(group) }
          end
          group
        end

        def scoped_id_from_parent
          if self.superclass == ExampleGroup
            Rex.world.next_top_level_index(file_path).to_s
          else
            "#{superclass.metadata[:scoped_id]}:#{superclass.next_runnable_index}"
          end
        end

        # Examples and child groups are numbered together, in the order they
        # are written.
        def next_runnable_index
          @runnable_index = (@runnable_index || 0) + 1
        end

        def define_example_group_method(name, extra_metadata = {})
          define_singleton_method(name) do |*arguments, &block|
            location = Metadata.location_from(caller_locations(1, 8))
            subclass(*arguments, location: location, extra_metadata: extra_metadata, &block)
          end
        end

        def define_example_method(name, extra_metadata = {})
          define_singleton_method(name) do |*arguments, &block|
            description = arguments.first.is_a?(Hash) ? nil : arguments.shift
            user_metadata = Metadata.build_hash_from(arguments).merge(extra_metadata)
            user_metadata[:skip] = "Not yet implemented" if block.nil? && !user_metadata.key?(:skip)
            location = Metadata.location_from(caller_locations(1, 8))
            example = Example.new(self, description, user_metadata, block, location)
            example.metadata[:scoped_id] = "#{metadata[:scoped_id]}:#{next_runnable_index}"
            example.metadata[:last_run_status] = Rex.configuration.last_run_statuses[example.id]
            examples << example
            example
          end
        end

        def before(*arguments, &block)
          scope, filter = Hooks.scope_and_metadata(arguments)
          hooks.register(:before, scope, filter, block, :append)
        end
        alias append_before before

        def prepend_before(*arguments, &block)
          scope, filter = Hooks.scope_and_metadata(arguments)
          hooks.register(:before, scope, filter, block, :prepend)
        end

        def after(*arguments, &block)
          scope, filter = Hooks.scope_and_metadata(arguments)
          hooks.register(:after, scope, filter, block, :prepend)
        end
        alias prepend_after after

        def append_after(*arguments, &block)
          scope, filter = Hooks.scope_and_metadata(arguments)
          hooks.register(:after, scope, filter, block, :append)
        end

        def around(*arguments, &block)
          scope, filter = Hooks.scope_and_metadata(arguments)
          hooks.register(:around, scope, filter, block)
        end

        def filtered_examples
          Rex.world.filtered_examples_for(self)
        end

        def descendant_filtered_examples
          filtered_examples + children.flat_map(&:descendant_filtered_examples)
        end

        # Runs this group's examples, then its child groups. Answers whether
        # every example passed.
        def run(reporter)
          return if Rex.world.wants_to_quit
          reporter.example_group_started(self)
          should_run_context_hooks = descendant_filtered_examples.any?
          begin
            run_before_context_hooks if should_run_context_hooks
            passed = run_examples(reporter)
            children_passed = ordering_strategy.order(children).map { |child| child.run(reporter) }.all?
            passed && children_passed
          rescue SkipDeclaredInExample => skipped
            descendant_filtered_examples.each { |example| example.skip_with_exception(reporter, skipped) }
            true
          rescue *Rex::Core::DANGEROUS_EXCEPTIONS
            raise
          rescue Exception => error
            descendant_filtered_examples.each { |example| example.fail_with_exception(reporter, error) }
            false
          ensure
            run_after_context_hooks if should_run_context_hooks
            reporter.example_group_finished(self)
          end
        end

        def ordering_strategy
          order = metadata.fetch(:order, :global)
          registry = Rex.configuration.ordering_registry
          registry.fetch(order) do
            warn <<~WARNING
              WARNING: Ignoring unknown ordering specified using `:order => #{order.inspect}` metadata.
                       Falling back to configured global ordering.
                       Unrecognized ordering specified at: #{location}
            WARNING
            registry.fetch(:global)
          end
        end

        def run_examples(reporter)
          ordering_strategy.order(filtered_examples).map do |example|
            next if Rex.world.wants_to_quit
            instance = new
            instance.__rex_context_ivars = before_context_ivars
            passed = example.run(instance, reporter)
            Rex.world.wants_to_quit = true if reporter.fail_fast_limit_met?
            passed
          end.all?
        end

        def before_context_ivars
          @before_context_ivars ||= superclass < ExampleGroup ? superclass.before_context_ivars.dup : {}
        end

        def run_before_context_hooks
          return if Rex.configuration.dry_run

          hooks_to_run = []
          hooks_to_run.concat(Rex.configuration.hooks.for(:before, :context).select { |hook| hook.applies_to?(metadata) }) if top_level?
          hooks_to_run.concat(hooks.for(:before, :context))
          return if hooks_to_run.empty?

          instance = new
          instance.__rex_context_ivars = before_context_ivars
          instance.__send__(:__rex_install_context_ivars)
          Rex.current_example = nil
          hooks_to_run.each { |hook| instance.instance_exec(&hook.block) }
          instance.instance_variables.each do |name|
            next if name.to_s.start_with?("@__")
            before_context_ivars[name] = instance.instance_variable_get(name)
          end
        end

        def run_after_context_hooks
          return if Rex.configuration.dry_run

          hooks_to_run = hooks.for(:after, :context).dup
          hooks_to_run.concat(Rex.configuration.hooks.for(:after, :context).select { |hook| hook.applies_to?(metadata) }) if top_level?
          return if hooks_to_run.empty?

          instance = new
          instance.__rex_context_ivars = before_context_ivars
          instance.__send__(:__rex_install_context_ivars)
          hooks_to_run.each do |hook|
            begin
              instance.instance_exec(&hook.block)
            rescue *Rex::Core::DANGEROUS_EXCEPTIONS
              raise
            rescue Exception => error
              Rex.world.reporter.notify_non_example_exception(error, "An error occurred in an `after(:context)` hook.")
            end
          end
        end
      end

      def self.build_description_from(parent_description = nil, my_description = nil)
        return parent_description.to_s unless my_description
        return my_description.to_s if parent_description.to_s == ""

        separator = description_separator(parent_description, my_description)
        parent_description.to_s + separator + my_description.to_s
      end

      def self.description_separator(parent_part, child_part)
        if parent_part.is_a?(Module) && /\A(?:#|::|\.)/.match?(child_part.to_s)
          ""
        else
          " "
        end
      end

      def self.full_description_for(metadata)
        description = metadata[:description]
        parent = metadata[:parent_example_group]
        return description unless parent

        separator = description_separator(parent[:description_args].last, metadata[:description_args].first)
        parent[:full_description] + separator + description
      end

      def self.described_class_from(description_args, parent_metadata)
        candidate = description_args.first
        return candidate unless candidate.nil? || candidate.is_a?(String)

        parent_metadata && parent_metadata[:described_class]
      end

      define_example_group_method :example_group
      define_example_group_method :describe
      define_example_group_method :context
      define_example_group_method :xdescribe, skip: "Temporarily skipped with xdescribe"
      define_example_group_method :xcontext, skip: "Temporarily skipped with xcontext"
      define_example_group_method :fdescribe, focus: true
      define_example_group_method :fcontext, focus: true

      define_example_method :example
      define_example_method :it
      define_example_method :specify
      define_example_method :focus, focus: true
      define_example_method :fexample, focus: true
      define_example_method :fit, focus: true
      define_example_method :fspecify, focus: true
      define_example_method :xexample, skip: "Temporarily skipped with xexample"
      define_example_method :xit, skip: "Temporarily skipped with xit"
      define_example_method :xspecify, skip: "Temporarily skipped with xspecify"
      define_example_method :skip, skip: true
      define_example_method :pending, pending: true

      attr_writer :__rex_context_ivars

      def initialize(inspect_output = nil)
        @__inspect_output = inspect_output || "an example"
      end

      def described_class
        self.class.described_class
      end

      # A name written bare that nothing defines is often a misspelled `let`,
      # so the error names the helpers declared around the example.
      def method_missing(name, *arguments, &block)
        super
      rescue NameError => error
        raise unless error.instance_of?(NameError) && error.name == name && arguments.empty?

        helpers = self.class.let_names
        raise if helpers.empty?

        hinted = NameError.new("#{error.message}\nHelpers declared nearby: #{helpers.join(', ')}", name, receiver: self)
        hinted.set_backtrace(error.backtrace)
        raise hinted, cause: nil
      end
      ruby2_keywords(:method_missing)

      def respond_to_missing?(name, include_private = false)
        super
      end

      def inspect
        "#<#{self.class} #{@__inspect_output}>"
      end

      private

      def __rex_install_context_ivars
        (@__rex_context_ivars || {}).each do |name, value|
          instance_variable_set(name, value)
        end
      end
    end
  end
end

module Rex
  # Every group is named as a constant here, so an error message names it,
  # the way RSpec names `RSpec::ExampleGroups::Calculator`.
  module ExampleGroups
    def self.assign_const(group)
      base_name = base_name_for(group)
      scope = constant_scope_for(group)
      name = disambiguate(base_name, scope)
      scope.const_set(name, group)
    end

    def self.constant_scope_for(group)
      scope = group.superclass
      scope == Rex::Core::ExampleGroup ? self : scope
    end

    def self.base_name_for(group)
      description = group.description.to_s
      return +"Anonymous" if description.empty?

      name = " " + description
      name = name.gsub(/[^0-9a-zA-Z]+([0-9a-zA-Z])/) { Regexp.last_match(1).upcase }
      name = name.lstrip.gsub(/\W/, "")
      name.sub(/\A([^A-Z]|\z)/, 'Nested\1')
    end

    def self.disambiguate(name, scope)
      return name unless scope.const_defined?(name, false)

      name += "_2"
      name = name.next while scope.const_defined?(name, false)
      name
    end
  end
end
