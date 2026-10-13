require "rex/core/backtrace_formatter"
require "rex/core/ruby_project"
require "rex/core/formatters/deprecation_formatter"
require "rex/core/output_wrapper"

module Rex
  module Core

    class Configuration
      include Rex::Core::Hooks

      Readers = Module.new
      include Readers

      class MustBeConfiguredBeforeExampleGroupsError < StandardError; end

      def self.define_reader(name)
        Readers.class_eval do
          remove_method name if method_defined?(name)
          attr_reader name
        end

        define_method(name) { value_for(name) { super() } }
      end

      def self.define_alias(name, alias_name)
        alias_method alias_name, name
        alias_method "#{alias_name}=", "#{name}="
        define_predicate alias_name
      end

      def self.define_predicate(name)
        define_method "#{name}?" do
          !!send(name)
        end
      end

      def self.add_setting(name, opts={})
        raise "Use the instance add_setting method if you want to set a default" if opts.key?(:default)
        attr_writer name
        add_read_only_setting name

        Array(opts[:alias_with]).each do |alias_name|
          define_alias(name, alias_name)
        end
      end

      def self.add_read_only_setting(name, opts={})
        raise "Use the instance add_setting method if you want to set a default" if opts.key?(:default)
        define_reader name
        define_predicate name
      end


      add_read_only_setting :default_path
      def default_path=(path)
        project_source_dirs << path
        @default_path = path
      end

      add_setting :drb

      add_setting :drb_port

      add_setting :error_stream

      def expose_dsl_globally?
        Core::DSL.exposed_globally?
      end

      def expose_dsl_globally=(value)
        if value
          Core::DSL.expose_globally!
          Core::SharedExampleGroup::TopLevelDSL.expose_globally!
        else
          Core::DSL.remove_globally!
          Core::SharedExampleGroup::TopLevelDSL.remove_globally!
        end
      end

      define_reader :deprecation_stream

      def deprecation_stream=(value)
        if @reporter && !value.equal?(@deprecation_stream)
          warn "RSpec's reporter has already been initialized with " \
            "#{deprecation_stream.inspect} as the deprecation stream, so your change to "\
            "`deprecation_stream` will be ignored. You should configure it earlier for " \
            "it to take effect, or use the `--deprecation-out` CLI option. " \
            "(Called from #{CallerFilter.first_non_rspec_line})"
        else
          @deprecation_stream = value
        end
      end

      define_reader :example_status_persistence_file_path

      def example_status_persistence_file_path=(value)
        @example_status_persistence_file_path = value
        clear_values_derived_from_example_status_persistence_file_path
      end

      define_reader :only_failures
      alias_method :only_failures?, :only_failures

      def only_failures_but_not_configured?
        only_failures? && !example_status_persistence_file_path
      end

      define_reader :fail_fast

      def fail_fast=(value)
        case value
        when true, 'true'
          @fail_fast = true
        when false, 'false', 0
          @fail_fast = false
        when nil
          @fail_fast = nil
        else
          @fail_fast = value.to_i

          if value.to_i == 0
            Rex.warning "Cannot set `RSpec.configuration.fail_fast`" \
              " to `#{value.inspect}`. Only `true`, `false`, `nil` and integers" \
              " are valid values."
            @fail_fast = true
          end
        end
      end

      add_setting :dry_run

      add_setting :failure_exit_code

      add_setting :error_exit_code

      add_setting :fail_if_no_examples

      define_reader :requires

      define_reader :libs

      define_reader :output_stream

      def output_stream=(value)
        if @reporter && !value.equal?(@output_stream)
          warn "RSpec's reporter has already been initialized with " \
            "#{output_stream.inspect} as the output stream, so your change to "\
            "`output_stream` will be ignored. You should configure it earlier for " \
            "it to take effect. (Called from #{CallerFilter.first_non_rspec_line})"
        else
          @output_stream = value
          output_wrapper.output = @output_stream
        end
      end

      define_reader :pattern

      def pattern=(value)
        update_pattern_attr :pattern, value
      end

      define_reader :exclude_pattern

      def exclude_pattern=(value)
        update_pattern_attr :exclude_pattern, value
      end

      add_setting :project_source_dirs

      attr_writer :profile_examples
      define_predicate :profile_examples

      add_setting :run_all_when_everything_filtered

      add_setting :success_color

      add_setting :pending_color

      add_setting :failure_color

      add_setting :default_color

      add_setting :fixed_color

      add_setting :detail_color

      add_setting :silence_filter_announcements

      def treat_symbols_as_metadata_keys_with_true_values=(_value)
        Rex.deprecate(
          "RSpec::Core::Configuration#treat_symbols_as_metadata_keys_with_true_values=",
          :message => "RSpec::Core::Configuration#treat_symbols_as_metadata_keys_with_true_values= " \
                      "is deprecated, it is now set to true as default and " \
                      "setting it to false has no effect."
        )
      end

      define_reader :shared_context_metadata_behavior
      def shared_context_metadata_behavior=(value)
        case value
        when :trigger_inclusion, :apply_to_host_groups
          @shared_context_metadata_behavior = value
        else
          raise ArgumentError, "Cannot set `RSpec.configuration." \
            "shared_context_metadata_behavior` to `#{value.inspect}`. Only " \
            "`:trigger_inclusion` and `:apply_to_host_groups` are valid values."
        end
      end

      add_setting :start_time

      add_setting :threadsafe

      add_setting :max_displayed_failure_line_count

      add_setting :full_cause_backtrace

      add_read_only_setting :pending_failure_output
      def pending_failure_output=(mode)
        raise ArgumentError,
              "`pending_failure_output` can be set to :full, :no_backtrace, " \
              "or :skip" unless [:full, :no_backtrace, :skip].include?(mode)
        @pending_failure_output = mode
      end

      attr_reader :bisect_runner
      def bisect_runner=(value)
        if @bisect_runner_class && value != @bisect_runner
          raise "`config.bisect_runner = #{value.inspect}` can no longer take " \
            "effect as the #{@bisect_runner.inspect} bisect runnner is already " \
            "in use. This config setting must be set in a file loaded by a " \
            "`--require` option (passed at the CLI or in a `.rspec` file) for " \
            "it to have any effect."
        end

        @bisect_runner = value
      end

      add_setting :tty
      attr_writer :files_to_run
      attr_accessor :filter_manager, :world
      attr_accessor :static_config_filter_manager
      attr_reader :backtrace_formatter, :ordering_manager, :loaded_spec_files


      def initialize
        @start_time = $_rspec_core_load_started_at || ::Rex::Core::Time.now
        @expectation_frameworks = []
        @include_modules = FilterableItemRepository::QueryOptimized.new(:any?)
        @extend_modules  = FilterableItemRepository::QueryOptimized.new(:any?)
        @prepend_modules = FilterableItemRepository::QueryOptimized.new(:any?)

        @bisect_runner = Rex::Support::RubyFeatures.fork_supported? ? :fork : :shell
        @bisect_runner_class = nil

        @before_suite_hooks = []
        @after_suite_hooks  = []

        @mock_framework = nil
        @files_or_directories_to_run = []
        @loaded_spec_files = Set.new
        @color = false
        @color_mode = :automatic
        @pattern = '**{,/*/**}/*_spec.rb'
        @exclude_pattern = ''
        @failure_exit_code = 1
        @error_exit_code = nil
        @fail_if_no_examples = false
        @spec_files_loaded = false

        @backtrace_formatter = BacktraceFormatter.new

        @default_path = 'spec'
        @project_source_dirs = %w[ spec lib app ]
        @deprecation_stream = $stderr
        @output_stream = $stdout
        @reporter = nil
        @reporter_buffer = nil
        @filter_manager = FilterManager.new
        @static_config_filter_manager = FilterManager.new
        @ordering_manager = Ordering::ConfigurationManager.new
        @preferred_options = {}
        @failure_color = :red
        @success_color = :green
        @pending_color = :yellow
        @default_color = :white
        @fixed_color = :blue
        @detail_color = :cyan
        @profile_examples = false
        @requires = []
        @libs = []
        @derived_metadata_blocks = FilterableItemRepository::QueryOptimized.new(:any?)
        @threadsafe = true
        @max_displayed_failure_line_count = 10
        @full_cause_backtrace = false
        @world = World::Null
        @shared_context_metadata_behavior = :trigger_inclusion
        @pending_failure_output = :full

        define_built_in_hooks
      end

      def force(hash)
        ordering_manager.force(hash)
        @preferred_options.merge!(hash)

        return unless hash.key?(:example_status_persistence_file_path)
        clear_values_derived_from_example_status_persistence_file_path
      end

      def reset
        @spec_files_loaded = false
        reset_reporter
      end

      def reset_reporter
        @reporter = nil
        @formatter_loader = nil
        @output_wrapper = nil
      end

      def reset_filters
        self.filter_manager = FilterManager.new
        filter_manager.include_only(
          Metadata.deep_hash_dup(static_config_filter_manager.inclusions.rules)
        )
        filter_manager.exclude_only(
          Metadata.deep_hash_dup(static_config_filter_manager.exclusions.rules)
        )
      end

      def add_setting(name, opts={})
        default = opts.delete(:default)
        (class << self; self; end).class_exec do
          add_setting(name, opts)
        end
        __send__("#{name}=", default) if default
      end

      def mock_framework
        if @mock_framework.nil?
          begin
            mock_with :rspec
          rescue LoadError
            mock_with :nothing
          end
        end
        @mock_framework
      end

      def mock_framework=(framework)
        mock_with framework
      end

      def backtrace_exclusion_patterns
        @backtrace_formatter.exclusion_patterns
      end

      def backtrace_exclusion_patterns=(patterns)
        @backtrace_formatter.exclusion_patterns = patterns
      end

      def backtrace_inclusion_patterns
        @backtrace_formatter.inclusion_patterns
      end

      def backtrace_inclusion_patterns=(patterns)
        @backtrace_formatter.inclusion_patterns = patterns
      end

      def filter_gems_from_backtrace(*gem_names)
        gem_names.each do |name|
          @backtrace_formatter.filter_gem(name)
        end
      end

      MOCKING_ADAPTERS = {
        :rspec    => :RSpec,
        :flexmock => :Flexmock,
        :rr       => :RR,
        :mocha    => :Mocha,
        :nothing  => :Null
      }

      def mock_with(framework)
        framework_module =
          if framework.is_a?(Module)
            framework
          else
            const_name = MOCKING_ADAPTERS.fetch(framework) do
              raise ArgumentError,
                    "Unknown mocking framework: #{framework.inspect}. " \
                    "Pass a module or one of #{MOCKING_ADAPTERS.keys.inspect}"
            end

            require "rex/core/mocking_adapters/#{const_name.to_s.downcase}"
            Rex::Core::MockingAdapters.const_get(const_name)
          end

        new_name, old_name = [framework_module, @mock_framework].map do |mod|
          mod.respond_to?(:framework_name) ? mod.framework_name : :unnamed
        end

        unless new_name == old_name
          assert_no_example_groups_defined(:mock_framework)
        end

        if block_given?
          raise "#{framework_module} must respond to `configuration` so that " \
                "mock_with can yield it." unless framework_module.respond_to?(:configuration)
          yield framework_module.configuration
        end

        @mock_framework = framework_module
      end

      def expectation_frameworks
        if @expectation_frameworks.empty?
          begin
            expect_with :rspec
          rescue LoadError
            expect_with Module.new
          end
        end
        @expectation_frameworks
      end

      def expectation_framework=(framework)
        expect_with(framework)
      end

      def expect_with(*frameworks)
        modules = frameworks.map do |framework|
          case framework
          when Module
            framework
          when :rspec
            require 'rspec/expectations'

            ::Rex::Expectations::MultipleExpectationsNotMetError.__send__(
              :include, MultipleExceptionError::InterfaceTag
            )

            ::Rex::Matchers
          when :test_unit
            require 'rspec/core/test_unit_assertions_adapter'
            ::Rex::Core::TestUnitAssertionsAdapter
          when :minitest
            require 'rspec/core/minitest_assertions_adapter'
            ::Rex::Core::MinitestAssertionsAdapter
          else
            raise ArgumentError, "#{framework.inspect} is not supported"
          end
        end

        if (modules - @expectation_frameworks).any?
          assert_no_example_groups_defined(:expect_with)
        end

        if block_given?
          raise "expect_with only accepts a block with a single argument. " \
                "Call expect_with #{modules.length} times, " \
                "once with each argument, instead." if modules.length > 1
          raise "#{modules.first} must respond to `configuration` so that " \
                "expect_with can yield it." unless modules.first.respond_to?(:configuration)
          yield modules.first.configuration
        end

        @expectation_frameworks.push(*modules)
      end

      def full_backtrace?
        @backtrace_formatter.full_backtrace?
      end

      def full_backtrace=(true_or_false)
        @backtrace_formatter.full_backtrace = true_or_false
      end

      def color
        value_for(:color) { @color }
      end

      def color_mode
        value_for(:color_mode) { @color_mode }
      end

      def color_enabled?(output=output_stream)
        case color_mode
        when :on then true
        when :off then false
        else
          output_to_tty?(output) || (color && tty?)
        end
      end

      attr_writer :color_mode

      attr_writer :color

      def libs=(libs)
        libs.map do |lib|
          @libs.unshift lib
          $LOAD_PATH.unshift lib
        end
      end

      def full_description=(description)
        filter_run :full_description => Regexp.union(*Array(description).map { |d| Regexp.new(d) })
      end

      def full_description
        filter.fetch :full_description, nil
      end

      def add_formatter(formatter, output=output_wrapper)
        formatter_loader.add(formatter, output)
      end
      alias_method :formatter=, :add_formatter

      def default_formatter
        formatter_loader.default_formatter
      end

      def default_formatter=(value)
        formatter_loader.default_formatter = value
      end

      def formatters
        formatter_loader.formatters.dup
      end

      def formatter_loader
        @formatter_loader ||= Formatters::Loader.new(Reporter.new(self))
      end

      class DeprecationReporterBuffer
        def initialize
          @calls = []
        end

        def deprecation(*args)
          @calls << args
        end

        def play_onto(reporter)
          @calls.each do |args|
            reporter.deprecation(*args)
          end
        end
      end

      def reporter
        @reporter_buffer || @reporter ||=
          begin
            @reporter_buffer = DeprecationReporterBuffer.new
            formatter_loader.prepare_default output_wrapper, deprecation_stream
            @reporter_buffer.play_onto(formatter_loader.reporter)
            @reporter_buffer = nil
            formatter_loader.reporter
          end
      end

      def profile_examples
        profile = value_for(:profile_examples) { @profile_examples }
        if profile && !profile.is_a?(Integer)
          10
        else
          profile
        end
      end

      def files_or_directories_to_run=(*files)
        files = files.flatten

        if (['rspec', 'metorex spec'].include?(command) || Runner.running_in_drb?) && default_path && files.empty?
          files << default_path
        end

        @files_or_directories_to_run = files
        @files_to_run = nil
      end

      def files_to_run
        @files_to_run ||= get_files_to_run(@files_or_directories_to_run)
      end

      def last_run_statuses
        @last_run_statuses ||= Hash.new(UNKNOWN_STATUS).tap do |statuses|
          if (path = example_status_persistence_file_path)
            begin
              ExampleStatusPersister.load_from(path).inject(statuses) do |hash, example|
                status = example[:status]
                status = UNKNOWN_STATUS unless VALID_STATUSES.include?(status)
                hash[example.fetch(:example_id)] = status
                hash
              end
            rescue SystemCallError => e
              Rex.warning "Could not read from #{path.inspect} (configured as " \
                            "`config.example_status_persistence_file_path`) due " \
                            "to a system error: #{e.inspect}. Please check that " \
                            "the config option is set to an accessible, valid " \
                            "file path", :call_site => nil
            end
          end
        end
      end

      UNKNOWN_STATUS = "unknown".freeze

      FAILED_STATUS = "failed".freeze

      PASSED_STATUS = "passed".freeze

      PENDING_STATUS = "pending".freeze

      VALID_STATUSES = [UNKNOWN_STATUS, FAILED_STATUS, PASSED_STATUS, PENDING_STATUS]

      def spec_files_with_failures
        @spec_files_with_failures ||= last_run_statuses.inject(Set.new) do |files, (id, status)|
          files << Example.parse_id(id).first if status == FAILED_STATUS
          files
        end.to_a
      end

      def alias_example_to(name, *args)
        extra_options = Metadata.build_hash_from(args)
        Rex::Core::ExampleGroup.define_example_method(name, extra_options)
      end

      def alias_example_group_to(new_name, *args)
        extra_options = Metadata.build_hash_from(args)
        Rex::Core::ExampleGroup.define_example_group_method(new_name, extra_options)
      end

      def alias_it_behaves_like_to(new_name, report_label='')
        Rex::Core::ExampleGroup.define_nested_shared_group_method(new_name, report_label)
      end
      alias_method :alias_it_should_behave_like_to, :alias_it_behaves_like_to

      def filter_run_including(*args)
        meta = Metadata.build_hash_from(args, :warn_about_example_group_filtering)
        filter_manager.include_with_low_priority meta
        static_config_filter_manager.include_with_low_priority Metadata.deep_hash_dup(meta)
      end
      alias_method :filter_run, :filter_run_including

      def filter_run_when_matching(*args)
        when_first_matching_example_defined(*args) do
          filter_run(*args)
        end
      end

      def inclusion_filter=(filter)
        meta = Metadata.build_hash_from([filter], :warn_about_example_group_filtering)
        filter_manager.include_only meta
      end

      alias_method :filter=, :inclusion_filter=

      def inclusion_filter
        filter_manager.inclusions
      end

      alias_method :filter, :inclusion_filter

      def filter_run_excluding(*args)
        meta = Metadata.build_hash_from(args, :warn_about_example_group_filtering)
        filter_manager.exclude_with_low_priority meta
        static_config_filter_manager.exclude_with_low_priority Metadata.deep_hash_dup(meta)
      end

      def exclusion_filter=(filter)
        meta = Metadata.build_hash_from([filter], :warn_about_example_group_filtering)
        filter_manager.exclude_only meta
      end

      def exclusion_filter
        filter_manager.exclusions
      end

      def include(mod, *filters)
        define_mixed_in_module(mod, filters, @include_modules, :include) do |group|
          safe_include(mod, group)
        end
      end

      def include_context(shared_group_name, *filters)
        shared_module = world.shared_example_group_registry.find([:main], shared_group_name)
        include shared_module, *filters
      end

      def extend(mod, *filters)
        define_mixed_in_module(mod, filters, @extend_modules, :extend) do |group|
          safe_extend(mod, group)
        end
      end

      if Rex::Support::RubyFeatures.module_prepends_supported?
        def prepend(mod, *filters)
          define_mixed_in_module(mod, filters, @prepend_modules, :prepend) do |group|
            safe_prepend(mod, group)
          end
        end
      end

      def configure_group(group)
        group.hooks.register_globals(group, hooks)

        configure_group_with group, @include_modules, :safe_include
        configure_group_with group, @extend_modules,  :safe_extend
        configure_group_with group, @prepend_modules, :safe_prepend
      end

      def configure_example(example, example_hooks)
        example_hooks.register_global_singleton_context_hooks(example, hooks)
        singleton_group = example.example_group_instance.singleton_class

        singleton_group.with_replaced_metadata(example.metadata) do
          modules = @include_modules.items_for(example.metadata)
          modules.each do |mod|
            safe_include(mod, example.example_group_instance.singleton_class)
          end

          MemoizedHelpers.define_helpers_on(singleton_group) unless modules.empty?
        end
      end

      def requires=(paths)
        directories = ['lib', default_path].select { |p| File.directory? p }
        Rex::Core::RubyProject.add_to_load_path(*directories)
        paths.each { |path|
          load_file_handling_errors(:require, path)
          @requires << path
        }
      end

      def in_project_source_dir_regex
        regexes = project_source_dirs.map do |dir|
          /\A#{Regexp.escape(File.expand_path(dir))}\//
        end

        Regexp.union(regexes)
      end

      def configure_mock_framework
        Rex::Core::ExampleGroup.__send__(:include, mock_framework)
        conditionally_disable_mocks_monkey_patching
      end

      def configure_expectation_framework
        expectation_frameworks.each do |framework|
          Rex::Core::ExampleGroup.__send__(:include, framework)
        end
        conditionally_disable_expectations_monkey_patching
      end

      def load_spec_files
        world.registered_example_group_files.each do |f|
          loaded_spec_files << f
        end

        files_to_run.uniq.each do |f|
          file = File.expand_path(f)
          load_file_handling_errors(:load, file)
          loaded_spec_files << file
        end

        @spec_files_loaded = true
      end

      DEFAULT_FORMATTER = lambda { |string| string }

      def format_docstrings(&block)
        @format_docstrings_block = block_given? ? block : DEFAULT_FORMATTER
      end

      def format_docstrings_block
        @format_docstrings_block ||= DEFAULT_FORMATTER
      end

      def self.delegate_to_ordering_manager(*methods)
        methods.each do |method|
          define_method method do |*args, &block|
            ordering_manager.__send__(method, *args, &block)
          end
        end
      end

      delegate_to_ordering_manager :seed=

      delegate_to_ordering_manager :seed

      delegate_to_ordering_manager :order=

      delegate_to_ordering_manager :register_ordering

      delegate_to_ordering_manager :seed_used?, :ordering_registry

      def warnings=(value)
        $VERBOSE = !!value
      end

      def warnings?
        $VERBOSE
      end

      RAISE_ERROR_WARNING_NOTIFIER = lambda { |message| raise message }

      def raise_on_warning=(value)
        if value
          Rex::Support.warning_notifier = RAISE_ERROR_WARNING_NOTIFIER
        else
          Rex::Support.warning_notifier = Rex::Support::DEFAULT_WARNING_NOTIFIER
        end
      end

      def expose_current_running_example_as(method_name)
        ExposeCurrentExample.module_exec do
          extend Rex::SharedContext
          let(method_name) { |ex| ex }
        end

        include ExposeCurrentExample
      end

      module ExposeCurrentExample; end

      def raise_errors_for_deprecations!
        self.deprecation_stream = Formatters::DeprecationFormatter::RaiseErrorStream.new
      end

      def disable_monkey_patching!
        self.expose_dsl_globally = false
        self.disable_monkey_patching = true
        conditionally_disable_mocks_monkey_patching
        conditionally_disable_expectations_monkey_patching
      end

      attr_accessor :disable_monkey_patching

      def define_derived_metadata(*filters, &block)
        meta = Metadata.build_hash_from(filters, :warn_about_example_group_filtering)
        @derived_metadata_blocks.append(block, meta)
      end

      def when_first_matching_example_defined(*filters)
        specified_meta = Metadata.build_hash_from(filters, :warn_about_example_group_filtering)

        callback = lambda do |example_or_group_meta|
          return unless example_or_group_meta.key?(:example_group)

          @derived_metadata_blocks.delete(callback, specified_meta)

          yield
        end

        @derived_metadata_blocks.append(callback, specified_meta)
      end

      def apply_derived_metadata_to(metadata)
        already_run_blocks = Set.new

        200.times do
          return if @derived_metadata_blocks.items_for(metadata).all? do |block|
            already_run_blocks.include?(block).tap do |skip_block|
              block.call(metadata) unless skip_block
              already_run_blocks << block
            end
          end
        end

        raise SystemStackError, "Attempted to recursively derive metadata indefinitely."
      end

      def before(scope=nil, *meta, &block)
        handle_suite_hook(scope, meta) do
          @before_suite_hooks << Hooks::BeforeHook.new(block, {})
        end || begin
          block.__id__

          add_hook_to_existing_matching_groups(meta, scope) { |g| g.before(scope, *meta, &block) }
          super(scope, *meta, &block)
        end
      end
      alias_method :append_before, :before

      def prepend_before(scope=nil, *meta, &block)
        handle_suite_hook(scope, meta) do
          @before_suite_hooks.unshift Hooks::BeforeHook.new(block, {})
        end || begin
          block.__id__

          add_hook_to_existing_matching_groups(meta, scope) { |g| g.prepend_before(scope, *meta, &block) }
          super(scope, *meta, &block)
        end
      end

      def after(scope=nil, *meta, &block)
        handle_suite_hook(scope, meta) do
          @after_suite_hooks.unshift Hooks::AfterHook.new(block, {})
        end || begin
          block.__id__

          add_hook_to_existing_matching_groups(meta, scope) { |g| g.after(scope, *meta, &block) }
          super(scope, *meta, &block)
        end
      end
      alias_method :prepend_after, :after

      def append_after(scope=nil, *meta, &block)
        handle_suite_hook(scope, meta) do
          @after_suite_hooks << Hooks::AfterHook.new(block, {})
        end || begin
          block.__id__

          add_hook_to_existing_matching_groups(meta, scope) { |g| g.append_after(scope, *meta, &block) }
          super(scope, *meta, &block)
        end
      end

      def around(scope=nil, *meta, &block)
        block.__id__

        add_hook_to_existing_matching_groups(meta, scope) { |g| g.around(scope, *meta, &block) }
        super(scope, *meta, &block)
      end

      def with_suite_hooks
        return yield if dry_run?

        begin
          Rex.current_scope = :before_suite_hook
          run_suite_hooks("a `before(:suite)` hook", @before_suite_hooks)
          yield
        ensure
          Rex.current_scope = :after_suite_hook
          run_suite_hooks("an `after(:suite)` hook", @after_suite_hooks)
          Rex.current_scope = :suite
        end
      end

      def hooks
        @hooks ||= HookCollections.new(self, FilterableItemRepository::QueryOptimized)
      end

      def on_example_group_definition(&block)
        on_example_group_definition_callbacks << block
      end

      def on_example_group_definition_callbacks
        @on_example_group_definition_callbacks ||= []
      end

      def bisect_runner_class
        @bisect_runner_class ||= begin
          case bisect_runner
          when :fork
            require "rex/core/bisect/fork_runner"
            Bisect::ForkRunner
          when :shell
            require "rex/core/bisect/shell_runner"
            Bisect::ShellRunner
          else
            raise "Unsupported value for `bisect_runner` (#{bisect_runner.inspect}). " \
                  "Only `:fork` and `:shell` are supported."
          end
        end
      end

    private

      def load_file_handling_errors(method, file)
        __send__(method, file)
      rescue LoadError => ex
        relative_file = Metadata.relative_path(file)
        suggestions = DidYouMean.new(relative_file).call
        reporter.notify_non_example_exception(ex, "An error occurred while loading #{relative_file}.#{suggestions}")
        Rex.world.wants_to_quit = true
      rescue SyntaxError => ex
        relative_file = Metadata.relative_path(file)
        reporter.notify_non_example_exception(
          ex,
          "While loading #{relative_file} a `raise SyntaxError` occurred, RSpec will now quit."
        )
        Rex.world.rspec_is_quitting = true
      rescue Support::AllExceptionsExceptOnesWeMustNotRescue => ex
        relative_file = Metadata.relative_path(file)
        reporter.notify_non_example_exception(ex, "An error occurred while loading #{relative_file}.")
        Rex.world.wants_to_quit = true
      rescue SystemExit => ex
        relative_file = Metadata.relative_path(file)
        reporter.notify_non_example_exception(
          ex,
          "While loading #{relative_file} an `exit` / `raise SystemExit` occurred, RSpec will now quit."
        )
        Rex.world.rspec_is_quitting = true
        raise ex
      end

      def handle_suite_hook(scope, meta)
        return nil unless scope == :suite

        unless meta.empty?
          Rex.warn_with "WARNING: `:suite` hooks do not support metadata since " \
                          "they apply to the suite as a whole rather than " \
                          "any individual example or example group that has metadata. " \
                          "The metadata you have provided (#{meta.inspect}) will be ignored."
        end

        yield
      end

      def run_suite_hooks(hook_description, hooks)
        context = SuiteHookContext.new(hook_description, reporter)

        hooks.each do |hook|
          begin
            hook.run(context)
          rescue Support::AllExceptionsExceptOnesWeMustNotRescue => ex
            context.set_exception(ex)

            break if hooks.equal?(@before_suite_hooks)
          end
        end
      end

      def get_files_to_run(paths)
        files = FlatMap.flat_map(paths_to_check(paths)) do |path|
          path = path.gsub(File::ALT_SEPARATOR, File::SEPARATOR) if File::ALT_SEPARATOR
          File.directory?(path) ? gather_directories(path) : extract_location(path)
        end.uniq

        return files unless only_failures?
        relative_files = files.map { |f| Metadata.relative_path(File.expand_path f) }
        intersection = (relative_files & spec_files_with_failures.to_a)
        intersection.empty? ? files : intersection
      end

      def paths_to_check(paths)
        return paths if pattern_might_load_specs_from_vendored_dirs?
        paths + [Dir.getwd]
      end

      def pattern_might_load_specs_from_vendored_dirs?
        pattern.split(File::SEPARATOR).first.include?('**')
      end

      def gather_directories(path)
        include_files = get_matching_files(path, pattern)
        exclude_files = get_matching_files(path, exclude_pattern)
        (include_files - exclude_files).uniq
      end

      def get_matching_files(path, pattern)
        raw_files = Dir[file_glob_from(path, pattern)]
        raw_files.map { |file| File.expand_path(file) }.sort
      end

      def file_glob_from(path, pattern)
        stripped = "{#{pattern.gsub(/\s*,\s*/, ',')}}"
        return stripped if pattern =~ /^(\.\/)?#{Regexp.escape path}/ || absolute_pattern?(pattern)
        File.join(path, stripped)
      end

      if Rex::Support::OS.windows?
        def absolute_pattern?(pattern)
          pattern =~ /\A[A-Z]:\\/ || windows_absolute_network_path?(pattern)
        end

        def windows_absolute_network_path?(pattern)
          return false unless ::File::ALT_SEPARATOR
          pattern.start_with?(::File::ALT_SEPARATOR + ::File::ALT_SEPARATOR)
        end
      else
        def absolute_pattern?(pattern)
          pattern.start_with?(File::Separator)
        end
      end

      def extract_location(path)
        match = /^(.*?)((?:\:\d+)+)$/.match(path)

        if match
          captures = match.captures
          path = captures[0]
          lines = captures[1][1..-1].split(":").map(&:to_i)
          filter_manager.add_location path, lines
        else
          path, scoped_ids = Example.parse_id(path)
          filter_manager.add_ids(path, scoped_ids.split(/\s*,\s*/)) if scoped_ids
        end

        return [] if path == default_path
        File.expand_path(path)
      end

      def command
        $0.split(File::SEPARATOR).last
      end

      def value_for(key)
        @preferred_options.fetch(key) { yield }
      end

      def define_built_in_hooks
        around(:example, :aggregate_failures => true) do |procsy|
          begin
            aggregate_failures(nil, :hide_backtrace => true, &procsy)
          rescue Support::AllExceptionsExceptOnesWeMustNotRescue => exception
            procsy.example.set_aggregate_failures_exception(exception)
          end
        end
      end

      def assert_no_example_groups_defined(config_option)
        return unless world.example_groups.any?

        raise MustBeConfiguredBeforeExampleGroupsError.new(
          "RSpec's #{config_option} configuration option must be configured before " \
          "any example groups are defined, but you have already defined a group."
        )
      end

      def output_wrapper
        @output_wrapper ||= OutputWrapper.new(output_stream)
      end

      def output_to_tty?(output=output_stream)
        output.respond_to?(:tty?) && output.tty?
      end

      def conditionally_disable_mocks_monkey_patching
        return unless disable_monkey_patching && rspec_mocks_loaded?

        Rex::Mocks.configuration.tap do |config|
          config.syntax = :expect
          config.patch_marshal_to_support_partial_doubles = false
        end
      end

      def conditionally_disable_expectations_monkey_patching
        return unless disable_monkey_patching && rspec_expectations_loaded?

        Rex::Expectations.configuration.syntax = :expect
      end

      def rspec_mocks_loaded?
        defined?(Rex::Mocks.configuration)
      end

      def rspec_expectations_loaded?
        defined?(Rex::Expectations.configuration)
      end

      def update_pattern_attr(name, value)
        if @spec_files_loaded
          Rex.warning "Configuring `#{name}` to #{value} has no effect since " \
                        "RSpec has already loaded the spec files."
        end

        instance_variable_set(:"@#{name}", value)
        @files_to_run = nil
      end

      def clear_values_derived_from_example_status_persistence_file_path
        @last_run_statuses = nil
        @spec_files_with_failures = nil
      end

      def configure_group_with(group, module_list, application_method)
        module_list.items_for(group.metadata).each do |mod|
          __send__(application_method, mod, group)
        end
      end

      def add_hook_to_existing_matching_groups(meta, scope, &block)
        if scope == :example || scope == :each || scope.nil?
          world.example_groups.each(&block)
        else
          meta = Metadata.build_hash_from(meta.dup)
          on_existing_matching_groups(meta, &block)
        end
      end

      def on_existing_matching_groups(meta)
        world.traverse_example_group_trees_until do |group|
          metadata_applies_to_group?(meta, group).tap do |applies|
            yield group if applies
          end
        end
      end

      def metadata_applies_to_group?(meta, group)
        meta.empty? || MetadataFilter.apply?(:any?, meta, group.metadata)
      end

      if Rex::Support::RubyFeatures.module_prepends_supported?
        def safe_prepend(mod, host)
          host.__send__(:prepend, mod) unless host < mod
        end
      end

      if RUBY_VERSION.to_f >= 1.9
        def safe_include(mod, host)
          host.__send__(:include, mod) unless host < mod
        end

        def safe_extend(mod, host)
          host.extend(mod) unless host.singleton_class < mod
        end
      else
        def safe_include(mod, host)
          host.__send__(:include, mod) unless host.included_modules.include?(mod)
        end

        def safe_extend(mod, host)
          host.extend(mod) unless (class << host; self; end).included_modules.include?(mod)
        end
      end

      def define_mixed_in_module(mod, filters, mod_list, config_method, &block)
        unless Module === mod
          raise TypeError, "`RSpec.configuration.#{config_method}` expects a module but got: #{mod.inspect}"
        end

        meta = Metadata.build_hash_from(filters, :warn_about_example_group_filtering)
        mod_list.append(mod, meta)
        on_existing_matching_groups(meta, &block)
      end
    end
  end
end
