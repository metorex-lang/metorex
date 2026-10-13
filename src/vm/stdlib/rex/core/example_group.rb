
module Rex
  module Core

    class ExampleGroup
      extend Hooks

      include MemoizedHelpers
      extend MemoizedHelpers::ClassMethods
      include Pending
      extend SharedExampleGroup

      def self.idempotently_define_singleton_method(name, &definition)
        (class << self; self; end).module_exec do
          remove_method(name) if method_defined?(name) && instance_method(name).owner == self
          define_method(name, &definition)
        end
      end


      def self.metadata
        @metadata ||= nil
      end

      def self.with_replaced_metadata(meta)
        orig_metadata = metadata
        @metadata = meta
        yield
      ensure
        @metadata = orig_metadata
      end

      def self.superclass_metadata
        @superclass_metadata ||= superclass.respond_to?(:metadata) ? superclass.metadata : nil
      end

      def self.delegate_to_metadata(*names)
        names.each do |name|
          idempotently_define_singleton_method(name) { metadata.fetch(name) }
        end
      end

      delegate_to_metadata :described_class, :file_path, :location

      def self.description
        description = metadata[:description]
        Rex.configuration.format_docstrings_block.call(description)
      end

      def described_class
        self.class.described_class
      end



      def self.define_example_method(name, extra_options={})
        idempotently_define_singleton_method(name) do |*all_args, &block|
          desc, *args = *all_args

          options = Metadata.build_hash_from(args)
          options.update(:skip => Rex::Core::Pending::NOT_YET_IMPLEMENTED) unless block
          options.update(extra_options)

          Rex::Core::Example.new(self, desc, options, block)
        end
      end

      define_example_method :example
      define_example_method :it
      define_example_method :specify

      define_example_method :focus,    :focus => true
      define_example_method :fexample, :focus => true
      define_example_method :fit,      :focus => true
      define_example_method :fspecify, :focus => true
      define_example_method :xexample, :skip => 'Temporarily skipped with xexample'
      define_example_method :xit,      :skip => 'Temporarily skipped with xit'
      define_example_method :xspecify, :skip => 'Temporarily skipped with xspecify'
      define_example_method :skip,     :skip => true
      define_example_method :pending,  :pending => true



      def self.define_example_group_method(name, metadata={})
        idempotently_define_singleton_method(name) do |*args, &example_group_block|
          thread_data = Rex::Support.thread_local_data
          top_level   = self == ExampleGroup

          registration_collection =
            if top_level
              if thread_data[:in_example_group]
                raise "Creating an isolated context from within a context is " \
                      "not allowed. Change `RSpec.#{name}` to `#{name}` or " \
                      "move this to a top-level scope."
              end

              thread_data[:in_example_group] = true
              Rex.world.example_groups
            else
              children
            end

          begin
            description = args.shift
            combined_metadata = metadata.dup
            combined_metadata.merge!(args.pop) if args.last.is_a? Hash
            args << combined_metadata

            subclass(self, description, args, registration_collection, &example_group_block)
          ensure
            thread_data.delete(:in_example_group) if top_level
          end
        end

        Rex::Core::DSL.expose_example_group_alias(name)
      end

      define_example_group_method :example_group

      define_example_group_method :describe

      define_example_group_method :context

      define_example_group_method :xdescribe, :skip => "Temporarily skipped with xdescribe"

      define_example_group_method :xcontext,  :skip => "Temporarily skipped with xcontext"

      define_example_group_method :fdescribe, :focus => true

      define_example_group_method :fcontext,  :focus => true



      def self.define_nested_shared_group_method(new_name, report_label="it should behave like")
        idempotently_define_singleton_method(new_name) do |name, *args, &customization_block|
          group = example_group("#{report_label} #{name}", :caller => (the_caller = caller)) do
            find_and_eval_shared("examples", name, the_caller.first, *args, &customization_block)
          end
          group.metadata[:shared_group_name] = name
          group
        end
      end

      define_nested_shared_group_method :it_behaves_like, "behaves like"
      define_nested_shared_group_method :it_should_behave_like

      def self.include_context(name, *args, &block)
        find_and_eval_shared("context", name, caller.first, *args, &block)
      end

      def self.include_examples(name, *args, &block)
        find_and_eval_shared("examples", name, caller.first, *args, &block)
      end

      def self.reset_memoized
        @descendant_filtered_examples = nil
        @_descendants = nil
        @parent_groups = nil
        @declaration_locations = nil
      end

      def self.add_example(example)
        reset_memoized
        examples << example
      end

      def self.remove_example(example)
        reset_memoized
        examples.delete example
      end

      def self.find_and_eval_shared(label, name, inclusion_location, *args, &customization_block)
        shared_module = Rex.world.shared_example_group_registry.find(parent_groups, name)

        unless shared_module
          raise ArgumentError, "Could not find shared #{label} #{name.inspect}"
        end

        shared_module.include_in(
          self, Metadata.relative_path(inclusion_location),
          args, customization_block
        )
      end


      def self.subclass(parent, description, args, registration_collection, &example_group_block)
        subclass = Class.new(parent)
        subclass.set_it_up(description, args, registration_collection, &example_group_block)
        subclass.module_exec(&example_group_block) if example_group_block

        MemoizedHelpers.define_helpers_on(subclass)

        subclass
      end

      def self.set_it_up(description, args, registration_collection, &example_group_block)
        ensure_example_groups_are_configured

        registration_collection << self

        @user_metadata = Metadata.build_hash_from(args)

        @metadata = Metadata::ExampleGroupHash.create(
          superclass_metadata, @user_metadata,
          superclass.method(:next_runnable_index_for),
          description, *args, &example_group_block
        )

        config = Rex.configuration
        config.apply_derived_metadata_to(@metadata)

        ExampleGroups.assign_const(self)

        @currently_executing_a_context_hook = false

        config.configure_group(self)
      end

      def self.examples
        @examples ||= []
      end

      def self.filtered_examples
        Rex.world.filtered_examples[self]
      end

      def self.descendant_filtered_examples
        @descendant_filtered_examples ||= filtered_examples +
          FlatMap.flat_map(children, &:descendant_filtered_examples)
      end

      def self.children
        @children ||= []
      end

      def self.traverse_tree_until(&block)
        return if yield self

        children.each do |child|
          child.traverse_tree_until(&block)
        end
      end

      def self.next_runnable_index_for(file)
        if self == ExampleGroup
          Rex.world.num_example_groups_defined_in(file) + 1
        else
          children.count + examples.count
        end
      end

      def self.descendants
        @_descendants ||= [self] + FlatMap.flat_map(children, &:descendants)
      end

      def self.parent_groups
        @parent_groups ||= ancestors.select { |a| a < Rex::Core::ExampleGroup }
      end

      def self.top_level?
        superclass == ExampleGroup
      end

      def self.ensure_example_groups_are_configured
        unless defined?(@@example_groups_configured)
          Rex.configuration.configure_mock_framework
          Rex.configuration.configure_expectation_framework
          @@example_groups_configured = true
        end
      end

      def self.before_context_ivars
        @before_context_ivars ||= {}
      end

      def self.store_before_context_ivars(example_group_instance)
        each_instance_variable_for_example(example_group_instance) do |ivar|
          before_context_ivars[ivar] = example_group_instance.instance_variable_get(ivar)
        end
      end

      def self.currently_executing_a_context_hook?
        @currently_executing_a_context_hook
      end

      def self.run_before_context_hooks(example_group_instance)
        set_ivars(example_group_instance, superclass_before_context_ivars)

        @currently_executing_a_context_hook = true

        ContextHookMemoized::Before.isolate_for_context_hook(example_group_instance) do
          hooks.run(:before, :context, example_group_instance)
        end
      ensure
        store_before_context_ivars(example_group_instance)
        @currently_executing_a_context_hook = false
      end

      if RUBY_VERSION.to_f >= 1.9
        def self.superclass_before_context_ivars
          superclass.before_context_ivars
        end
      else
        def self.superclass_before_context_ivars
          if superclass.respond_to?(:before_context_ivars)
            superclass.before_context_ivars
          else
            ancestors.find { |a| a.respond_to?(:before_context_ivars) }.before_context_ivars
          end
        end
      end

      def self.run_after_context_hooks(example_group_instance)
        set_ivars(example_group_instance, before_context_ivars)

        @currently_executing_a_context_hook = true

        ContextHookMemoized::After.isolate_for_context_hook(example_group_instance) do
          hooks.run(:after, :context, example_group_instance)
        end
      ensure
        before_context_ivars.clear
        @currently_executing_a_context_hook = false
      end

      def self.run(reporter=Rex::Core::NullReporter)
        return if Rex.world.wants_to_quit
        reporter.example_group_started(self)

        should_run_context_hooks = descendant_filtered_examples.any?
        begin
          Rex.current_scope = :before_context_hook
          run_before_context_hooks(new_instance_for('before(:context) hook')) if should_run_context_hooks
          result_for_this_group = run_examples(reporter)
          results_for_descendants = ordering_strategy.order(children).map { |child| child.run(reporter) }.all?
          result_for_this_group && results_for_descendants
        rescue Pending::SkipDeclaredInExample => ex
          for_filtered_examples(reporter) { |example| example.skip_with_exception(reporter, ex) }
          true
        rescue Support::AllExceptionsExceptOnesWeMustNotRescue => ex
          for_filtered_examples(reporter) { |example| example.fail_with_exception(reporter, ex) }
          Rex.world.wants_to_quit = true if reporter.fail_fast_limit_met?
          false
        ensure
          Rex.current_scope = :after_context_hook
          run_after_context_hooks(new_instance_for('after(:context) hook')) if should_run_context_hooks
          reporter.example_group_finished(self)
        end
      end

      def self.ordering_strategy
        order = metadata.fetch(:order, :global)
        registry = Rex.configuration.ordering_registry

        registry.fetch(order) do
          warn <<-WARNING.gsub(/^ +\|/, '')
            |WARNING: Ignoring unknown ordering specified using `:order => #{order.inspect}` metadata.
            |         Falling back to configured global ordering.
            |         Unrecognized ordering specified at: #{location}
          WARNING

          registry.fetch(:global)
        end
      end

      # A fast path: the instance built natively, as `new` builds it,
      # unless a group, a module, or a redefinition gives it initializers
      # of its own.
      def self.new_instance_for(inspect_output)
        threadsafe = Rex.configuration.threadsafe?
        memo_class = threadsafe ? MemoizedHelpers::ThreadsafeMemoized : MemoizedHelpers::NonThreadSafeMemoized
        __new_example_group__(self, inspect_output, memo_class,
                              threadsafe ? Support::ReentrantMutex : nil, ExampleGroup) ||
          new(inspect_output)
      end
      private_class_method :new_instance_for

      def self.run_examples(reporter)
        ordering_strategy.order(filtered_examples).map do |example|
          next if Rex.world.wants_to_quit
          instance = new_instance_for(example.inspect_output)
          set_ivars(instance, before_context_ivars)
          succeeded = example.run(instance, reporter)
          if !succeeded && reporter.fail_fast_limit_met?
            Rex.world.wants_to_quit = true
          end
          succeeded
        end.all?
      end

      def self.for_filtered_examples(reporter, &block)
        filtered_examples.each(&block)

        children.each do |child|
          reporter.example_group_started(child)
          child.for_filtered_examples(reporter, &block)
          reporter.example_group_finished(child)
        end
        false
      end

      def self.declaration_locations
        @declaration_locations ||= [Metadata.location_tuple_from(metadata)] +
          examples.map { |e| Metadata.location_tuple_from(e.metadata) } +
          FlatMap.flat_map(children, &:declaration_locations)
      end

      def self.id
        Metadata.id_from(metadata)
      end

      def self.top_level_description
        parent_groups.last.description
      end

      def self.set_ivars(instance, ivars)
        ivars.each { |name, value| instance.instance_variable_set(name, value) }
      end

      if RUBY_VERSION.to_f < 1.9
        INSTANCE_VARIABLE_TO_IGNORE = '@__inspect_output'.freeze
      else
        INSTANCE_VARIABLE_TO_IGNORE = :@__inspect_output
      end

      def self.each_instance_variable_for_example(group)
        group.instance_variables.each do |ivar|
          yield ivar unless ivar == INSTANCE_VARIABLE_TO_IGNORE
        end
      end

      def initialize(inspect_output=nil)
        @__inspect_output = inspect_output || '(no description provided)'
        super()
      end

      def inspect
        "#<#{self.class} #{@__inspect_output}>"
      end

      unless method_defined?(:singleton_class)
        def singleton_class
          class << self; self; end
        end
      end

      def self.update_inherited_metadata(updates)
        metadata.update(updates) do |key, existing_group_value, new_inherited_value|
          @user_metadata.key?(key) ? existing_group_value : new_inherited_value
        end

        Rex.configuration.configure_group(self)
        examples.each { |ex| ex.update_inherited_metadata(updates) }
        children.each { |group| group.update_inherited_metadata(updates) }
      end

      WrongScopeError = Class.new(NoMethodError)

      def self.method_missing(name, *args)
        if method_defined?(name)
          raise WrongScopeError,
                "`#{name}` is not available on an example group (e.g. a " \
                "`describe` or `context` block). It is only available from " \
                "within individual examples (e.g. `it` blocks) or from " \
                "constructs that run in the scope of an example (e.g. " \
                "`before`, `let`, etc)."
        end

        super
      end
      private_class_method :method_missing

    private

      def method_missing(name, *args)
        if self.class.respond_to?(name)
          raise WrongScopeError,
                "`#{name}` is not available from within an example (e.g. an " \
                "`it` block) or from constructs that run in the scope of an " \
                "example (e.g. `before`, `let`, etc). It is only available " \
                "on an example group (e.g. a `describe` or `context` block)."
        end

        begin
          super(name, *args)
        rescue NameError => error
          raise unless error.instance_of?(NameError) && error.name == name && args.empty?

          helpers = self.class.parent_groups.flat_map do |group|
            next [] unless group.const_defined?(:LetDefinitions, false)

            group.const_get(:LetDefinitions, false).instance_methods(false)
          end.uniq
          raise if helpers.empty?

          hinted = NameError.new("#{error.message}\nHelpers declared nearby: #{helpers.join(', ')}", name, receiver: self)
          hinted.set_backtrace(error.backtrace)
          raise hinted, cause: nil
        end
      end
      ruby2_keywords :method_missing if respond_to?(:ruby2_keywords, true)
    end

    class AnonymousExampleGroup < ExampleGroup
      def self.metadata
        {}
      end
    end

    class SharedExampleGroupInclusionStackFrame
      attr_reader :shared_group_name
      attr_reader :inclusion_location

      def initialize(shared_group_name, inclusion_location)
        @shared_group_name  = shared_group_name
        @inclusion_location = inclusion_location
      end

      def formatted_inclusion_location
        @formatted_inclusion_location ||= begin
          Rex.configuration.backtrace_formatter.backtrace_line(
            inclusion_location.sub(/(:\d+):in .+$/, '\1')
          )
        end
      end

      def description
        @description ||= "Shared Example Group: #{shared_group_name.inspect} " \
          "called from #{formatted_inclusion_location}"
      end

      def self.current_backtrace
        shared_example_group_inclusions.reverse
      end

      def self.with_frame(name, location)
        current_stack = shared_example_group_inclusions
        if current_stack.any? { |frame| frame.shared_group_name == name }
          raise ArgumentError, "can't include shared examples recursively"
        else
          current_stack << new(name, location)
          yield
        end
      ensure
        current_stack.pop
      end

      def self.shared_example_group_inclusions
        Rex::Support.thread_local_data[:shared_example_group_inclusions] ||= []
      end
    end
  end

  module ExampleGroups
    extend Support::RecursiveConstMethods

    def self.assign_const(group)
      base_name   = base_name_for(group)
      const_scope = constant_scope_for(group)
      name        = disambiguate(base_name, const_scope)

      const_scope.const_set(name, group)
    end

    def self.constant_scope_for(group)
      const_scope = group.superclass
      const_scope = self if const_scope == ::Rex::Core::ExampleGroup
      const_scope
    end

    def self.remove_all_constants
      constants.each do |constant|
        __send__(:remove_const, constant)
      end
    end

    def self.base_name_for(group)
      return "Anonymous".dup if group.description.empty?

      name = ' ' + group.description
      name.gsub!(/[^0-9a-zA-Z]+([0-9a-zA-Z])/) do
        match = ::Regexp.last_match[1]
        match.upcase!
        match
      end

      name.lstrip!
      name.gsub!(/\W/, ''.freeze)

      name.gsub!(/\A([^A-Z]|\z)/, 'Nested\1'.freeze)

      name
    end

    if RUBY_VERSION == '1.9.2'
      class << self
        alias _base_name_for base_name_for
        def base_name_for(group)
          _base_name_for(group) + '_'
        end
      end
      private_class_method :_base_name_for
    end

    def self.disambiguate(name, const_scope)
      return name unless const_defined_on?(const_scope, name)

      name << "_2"
      name.next! while const_defined_on?(const_scope, name)
      name
    end
  end
end
