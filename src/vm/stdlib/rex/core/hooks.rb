module Rex
  module Core
    module Hooks
      def before(*args, &block)
        hooks.register :append, :before, *args, &block
      end

      alias_method :append_before, :before

      def prepend_before(*args, &block)
        hooks.register :prepend, :before, *args, &block
      end

      def after(*args, &block)
        hooks.register :prepend, :after, *args, &block
      end

      alias_method :prepend_after, :after

      def append_after(*args, &block)
        hooks.register :append, :after, *args, &block
      end

      def around(*args, &block)
        hooks.register :prepend, :around, *args, &block
      end

      def hooks
        @hooks ||= HookCollections.new(self, FilterableItemRepository::UpdateOptimized)
      end

      Hook = Struct.new(:block, :options)

      class BeforeHook < Hook
        def run(example)
          example.instance_exec(example, &block)
        end
      end

      class AfterHook < Hook
        def run(example)
          example.instance_exec(example, &block)
        rescue Support::AllExceptionsExceptOnesWeMustNotRescue => ex
          example.set_exception(ex)
        end
      end

      class AfterContextHook < Hook
        def run(example)
          example.instance_exec(example, &block)
        rescue Support::AllExceptionsExceptOnesWeMustNotRescue => e
          Rex.configuration.reporter.notify_non_example_exception(e, "An error occurred in an `after(:context)` hook.")
        end
      end

      class AroundHook < Hook
        def execute_with(example, procsy)
          example.instance_exec(procsy, &block)
          return if procsy.executed?
          Pending.mark_skipped!(example,
                                "#{hook_description} did not execute the example")
        end

        if Proc.method_defined?(:source_location)
          def hook_description
            "around hook at #{Metadata.relative_path(block.source_location.join(':'))}"
          end
        else
          def hook_description
            "around hook"
          end
        end
      end

      class HookCollections
        def initialize(owner, filterable_item_repo_class)
          @owner                      = owner
          @filterable_item_repo_class = filterable_item_repo_class
          @before_example_hooks       = nil
          @after_example_hooks        = nil
          @before_context_hooks       = nil
          @after_context_hooks        = nil
          @around_example_hooks       = nil
        end

        def register_globals(host, globals)
          parent_groups = host.parent_groups

          process(host, parent_groups, globals, :before, :example, &:options)
          process(host, parent_groups, globals, :after,  :example, &:options)
          process(host, parent_groups, globals, :around, :example, &:options)

          process(host, parent_groups, globals, :before, :context, &:options)
          process(host, parent_groups, globals, :after,  :context, &:options)
        end

        def register_global_singleton_context_hooks(example, globals)
          parent_groups = example.example_group.parent_groups

          process(example, parent_groups, globals, :before, :context) { {} }
          process(example, parent_groups, globals, :after,  :context) { {} }
        end

        def register(prepend_or_append, position, *args, &block)
          scope, options = scope_and_options_from(*args)

          if scope == :suite
            Rex.warn_with "WARNING: `#{position}(:suite)` hooks are only supported on " \
                            "the RSpec configuration object. This " \
                            "`#{position}(:suite)` hook, registered on an example " \
                            "group, will be ignored."
            return
          elsif scope == :context && position == :around
            Rex.warn_with "WARNING: `around(:context)` hooks are not supported and " \
                            "behave like `around(:example)`."
          end

          hook = HOOK_TYPES[position][scope].new(block, options)
          ensure_hooks_initialized_for(position, scope).__send__(prepend_or_append, hook, options)
        end

        def run(position, scope, example_or_group)
          return if Rex.configuration.dry_run?

          if scope == :context
            unless example_or_group.class.metadata[:skip]
              run_owned_hooks_for(position, :context, example_or_group)
            end
          else
            case position
            when :before then run_example_hooks_for(example_or_group, :before, :reverse_each)
            when :after  then run_example_hooks_for(example_or_group, :after,  :each)
            when :around then run_around_example_hooks_for(example_or_group) { yield }
            end
          end
        end

        SCOPES = [:example, :context]

        SCOPE_ALIASES = { :each => :example, :all => :context }

        HOOK_TYPES = {
          :before => Hash.new { BeforeHook },
          :after  => Hash.new { AfterHook  },
          :around => Hash.new { AroundHook }
        }

        HOOK_TYPES[:after][:context] = AfterContextHook

      protected

        EMPTY_HOOK_ARRAY = [].freeze

        def matching_hooks_for(position, scope, example_or_group)
          repository = hooks_for(position, scope) { return EMPTY_HOOK_ARRAY }

          metadata = case example_or_group
                     when ExampleGroup then example_or_group.class.metadata
                     else example_or_group.metadata
                     end

          repository.items_for(metadata)
        end

        def all_hooks_for(position, scope)
          hooks_for(position, scope) { return EMPTY_HOOK_ARRAY }.items_and_filters.map(&:first)
        end

        def run_owned_hooks_for(position, scope, example_or_group)
          matching_hooks_for(position, scope, example_or_group).each do |hook|
            hook.run(example_or_group)
          end
        end

        # The example hooks registered here, with the metadata each is
        # filtered by.
        def example_hook_entries(position)
          hooks_for(position, :example) { return EMPTY_HOOK_ARRAY }.items_and_filters
        end

        def processable_hooks_for(position, scope, host)
          if scope == :example
            all_hooks_for(position, scope)
          else
            matching_hooks_for(position, scope, host)
          end
        end

      private

        def hooks_for(position, scope)
          if position == :before
            scope == :example ? @before_example_hooks : @before_context_hooks
          elsif position == :after
            scope == :example ? @after_example_hooks : @after_context_hooks
          else
            @around_example_hooks
          end || yield
        end

        def ensure_hooks_initialized_for(position, scope)
          if position == :before
            if scope == :example
              @before_example_hooks ||= @filterable_item_repo_class.new(:all?)
            else
              @before_context_hooks ||= @filterable_item_repo_class.new(:all?)
            end
          elsif position == :after
            if scope == :example
              @after_example_hooks ||= @filterable_item_repo_class.new(:all?)
            else
              @after_context_hooks ||= @filterable_item_repo_class.new(:all?)
            end
          else
            @around_example_hooks ||= @filterable_item_repo_class.new(:all?)
          end
        end

        def process(host, parent_groups, globals, position, scope)
          hooks_to_process = globals.processable_hooks_for(position, scope, host)
          return if hooks_to_process.empty?

          hooks_to_process -= FlatMap.flat_map(parent_groups) do |group|
            group.hooks.all_hooks_for(position, scope)
          end
          return if hooks_to_process.empty?

          repository = ensure_hooks_initialized_for(position, scope)
          hooks_to_process.each { |hook| repository.append hook, (yield hook) }
        end

        def scope_and_options_from(*args)
          return :suite if args.first == :suite
          scope = extract_scope_from(args)
          meta  = Metadata.build_hash_from(args, :warn_about_example_group_filtering)
          return scope, meta
        end

        def extract_scope_from(args)
          if known_scope?(args.first)
            normalized_scope_for(args.shift)
          elsif args.any? { |a| a.is_a?(Symbol) }
            error_message = "You must explicitly give a scope " \
              "(#{SCOPES.join(", ")}) or scope alias " \
              "(#{SCOPE_ALIASES.keys.join(", ")}) when using symbols as " \
              "metadata for a hook."
            raise ArgumentError.new error_message
          else
            :example
          end
        end

        def known_scope?(scope)
          SCOPES.include?(scope) || SCOPE_ALIASES.keys.include?(scope)
        end

        def normalized_scope_for(scope)
          SCOPE_ALIASES[scope] || scope
        end

        def run_example_hooks_for(example, position, each_method)
          if (entries = flattened_example_hook_entries(example, position))
            metadata = example.metadata
            entries.each do |hook, filters|
              hook.run(example) if filters.empty? || MetadataFilter.apply?(:all?, filters, metadata)
            end
            return
          end

          owner_parent_groups.__send__(each_method) do |group|
            group.hooks.run_owned_hooks_for(position, :example, example)
          end
        end

        # A fast path: the example hooks of the example's group and the
        # groups around it, in the order they run, built once and kept until
        # a hook is added anywhere, with this collection's own put on the
        # end they run from. An example's hooks are this collection's when
        # it belongs to the singleton class of the example's group instance.
        # nil when that is not so, or when the guard does not hold.
        def flattened_example_hook_entries(example, position)
          groups = owner_parent_groups
          group = groups[1]
          return nil unless group && group.equal?(example.example_group) && groups.first.singleton_class?

          return nil unless Rex.configuration.fast_paths?

          flattened = FastPaths::FLATTENED_HOOKS[group] ||= { receivers: groups.drop(1).map(&:hooks) }
          state = __method_state__
          unless flattened[:state] == state
            flattened[:state] = state
            flattened[:held] = FastPaths.hooks_guard.holds?(flattened[:receivers])
          end
          return nil unless flattened[:held]

          changes = FilterableItemRepository.changes
          unless flattened[:changes] == changes
            flattened[:changes] = changes
            parents = groups.drop(1)
            flattened[:before] = parents.reverse.flat_map { |each_group| each_group.hooks.example_hook_entries(:before) }
            flattened[:after] = parents.flat_map { |each_group| each_group.hooks.example_hook_entries(:after) }
            flattened[:around] = parents.flat_map { |each_group| each_group.hooks.example_hook_entries(:around) }
          end
          own = example_hook_entries(position)
          return flattened[position] if own.empty?

          position == :before ? flattened[position] + own : own + flattened[position]
        end

        def run_around_example_hooks_for(example)
          hooks = if (entries = flattened_example_hook_entries(example, :around))
                    metadata = example.metadata
                    entries.filter_map do |hook, filters|
                      hook if filters.empty? || MetadataFilter.apply?(:all?, filters, metadata)
                    end
                  else
                    FlatMap.flat_map(owner_parent_groups) do |group|
                      group.hooks.matching_hooks_for(:around, :example, example)
                    end
                  end

          return yield if hooks.empty?

          initial_procsy = Example::Procsy.new(example) { yield }
          hooks.inject(initial_procsy) do |procsy, around_hook|
            procsy.wrap { around_hook.execute_with(example, procsy) }
          end.call
        end

        if respond_to?(:singleton_class) && singleton_class.ancestors.include?(singleton_class)
          def owner_parent_groups
            @owner.parent_groups
          end
        else
          def owner_parent_groups
            @owner_parent_groups ||= [@owner] + @owner.parent_groups
          end
        end
      end
    end
  end
end
