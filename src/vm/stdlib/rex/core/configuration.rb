module Rex
  module Core
    # The settings a suite changes through `Rex.configure`, and the hooks and
    # modules it registers for every group.
    class Configuration
      attr_accessor :pattern, :exclude_pattern, :fail_fast,
                    :failure_exit_code, :error_exit_code, :color_mode,
                    :dry_run, :default_formatter,
                    :output_stream, :error_stream,
                    :silence_filter_announcements,
                    :only_failures,
                    :run_all_when_everything_filtered,
                    :libs, :requires,
                    :shared_context_metadata_behavior, :max_displayed_failure_line_count,
                    :threadsafe, :detail_color, :failure_color, :success_color,
                    :pending_color, :default_color, :fixed_color

      attr_reader :hooks, :inclusions, :extensions, :prependings,
                  :inclusion_filter, :exclusion_filter, :formatters

      def initialize
        @default_path = "spec"
        @pattern = "**{,/*/**}/*_spec.rb"
        @exclude_pattern = ""
        @fail_fast = false
        @failure_exit_code = 1
        @error_exit_code = nil
        @color_mode = :automatic
        @dry_run = false
        @ordering_manager = Ordering::ConfigurationManager.new
        @default_formatter = "progress"
        @output_stream = $stdout
        @error_stream = $stderr
        @deprecation_stream = $stderr
        @backtrace_formatter = BacktraceFormatter.new
        @silence_filter_announcements = false
        @example_status_persistence_file_path = nil
        @run_all_when_everything_filtered = false
        @profile_examples = false
        @expose_dsl_globally = true
        @libs = []
        @requires = []
        @shared_context_metadata_behavior = :trigger_inclusion
        @max_displayed_failure_line_count = 10
        @threadsafe = true
        @detail_color = :cyan
        @failure_color = :red
        @success_color = :green
        @pending_color = :yellow
        @default_color = :white
        @fixed_color = :blue
        @hooks = Hooks::Collection.new
        @inclusions = []
        @extensions = []
        @prependings = []
        @inclusion_filter = {}
        @exclusion_filter = {}
        @formatters = []
        @expectation_syntax = [:expect]
      end

      def before(*arguments, &block)
        register_hook(:before, arguments, block, :append)
      end
      alias append_before before

      def prepend_before(*arguments, &block)
        register_hook(:before, arguments, block, :prepend)
      end

      def after(*arguments, &block)
        register_hook(:after, arguments, block, :prepend)
      end
      alias prepend_after after

      def append_after(*arguments, &block)
        register_hook(:after, arguments, block, :append)
      end

      def around(*arguments, &block)
        register_hook(:around, arguments, block, :append)
      end

      def register_hook(kind, arguments, block, position)
        scope, metadata = Hooks.scope_and_metadata(arguments.dup)
        @hooks.register(kind, scope, metadata, block, position)
      end
      private :register_hook

      def include(mod, *metadata)
        @inclusions << [mod, Metadata.build_hash_from(metadata)]
      end

      def extend(mod, *metadata)
        @extensions << [mod, Metadata.build_hash_from(metadata)]
      end

      def prepend(mod, *metadata)
        @prependings << [mod, Metadata.build_hash_from(metadata)]
      end

      # A filter the configuration sets has lower priority than one from the
      # command line, so it goes ahead of the ones already there, as RSpec
      # orders them.
      def filter_run_including(*arguments)
        @inclusion_filter = Metadata.build_hash_from(arguments).merge(@inclusion_filter)
      end
      alias filter_run filter_run_including

      def filter_run_excluding(*arguments)
        @exclusion_filter = Metadata.build_hash_from(arguments).merge(@exclusion_filter)
      end

      def filter_run_when_matching(*arguments)
        @when_matching_filter ||= {}
        @when_matching_filter.update(Metadata.build_hash_from(arguments))
      end

      def when_matching_filter
        @when_matching_filter || {}
      end

      # A formatter named, given as a class, or given as an instance. Once
      # the run's reporter is set up, it is attached right away.
      def add_formatter(formatter, output = nil)
        @formatters << [formatter, output]
        reporter = Rex.world.reporter
        reporter.add_formatter(Formatters.build(formatter, output, self)) if reporter&.set_up?
      end
      alias formatter= add_formatter

      def color
        @color_mode == :on || (@color_mode == :automatic && output_stream.respond_to?(:tty?) && output_stream.tty?)
      end

      def color=(value)
        @color_mode = value ? :on : :off
      end

      def color_enabled?(stream = output_stream)
        case @color_mode
        when :on then true
        when :off then false
        else (stream.respond_to?(:tty?) && stream.tty?) || tty?
        end
      end

      def expect_with(*frameworks)
        @expectation_frameworks = frameworks.map { |framework| framework == :rspec ? Rex::Matchers : framework }
        yield ExpectationConfiguration.new(self) if block_given?
      end

      def mock_with(framework)
        @mock_framework = framework == :rspec ? Rex::Mocks : framework
        yield Rex::Mocks.configuration if block_given?
      end

      def mock_framework
        @mock_framework || Rex::Mocks
      end

      def mock_framework=(framework)
        mock_with(framework)
      end

      def expectation_frameworks
        @expectation_frameworks.nil? || @expectation_frameworks.empty? ? [Rex::Matchers] : @expectation_frameworks
      end

      def expectation_framework=(framework)
        expect_with(framework)
      end

      def expectation_syntax
        @expectation_syntax
      end

      def expectation_syntax=(syntaxes)
        @expectation_syntax = Array(syntaxes)
        Expectations::Syntax.enable_should if @expectation_syntax.include?(:should)
        Expectations::Syntax.disable_should unless @expectation_syntax.include?(:should)
      end

      # A setting RSpec 2 needed, which RSpec 3 always behaves as.
      def treat_symbols_as_metadata_keys_with_true_values=(_value)
        Rex.deprecate(
          "RSpec::Core::Configuration#treat_symbols_as_metadata_keys_with_true_values=",
          message: "RSpec::Core::Configuration#treat_symbols_as_metadata_keys_with_true_values= " \
                   "is deprecated, it is now set to true as default and " \
                   "setting it to false has no effect."
        )
      end

      def disable_monkey_patching!
        self.expose_dsl_globally = false
        self.disable_monkey_patching = true
        self.expectation_syntax = [:expect]
      end

      def define_derived_metadata(*filters, &block)
        @derived_metadata_blocks ||= []
        @derived_metadata_blocks << [Metadata.build_hash_from(filters), block]
      end

      def derived_metadata_blocks
        @derived_metadata_blocks || []
      end

      def requires=(paths)
        @requires = paths
      end

      def files_or_directories_to_run=(paths)
        @files_or_directories_to_run = paths
      end

      def files_or_directories_to_run
        @files_or_directories_to_run || []
      end

      def loaded_spec_files
        @loaded_spec_files ||= []
      end

      # The directories whose files count as the project's own when a
      # failure looks for the line to quote.
      def project_source_dirs
        @project_source_dirs ||= %w[spec lib app]
      end
      attr_writer :project_source_dirs

      attr_reader :default_path, :deprecation_stream, :expose_dsl_globally

      # Ruby's own warnings, which `config.warnings = true` turns on.
      def warnings=(value)
        $VERBOSE = !!value
      end

      def warnings?
        $VERBOSE
      end
      alias warnings warnings?

      # Puts `describe` and its siblings on `main` and every Module, or takes
      # them off again.
      def expose_dsl_globally=(value)
        @expose_dsl_globally = value
        value ? DSL.expose_globally! : DSL.remove_globally!
      end

      # RSpec builds its reporter after the spec files load, so a stream set
      # in one still takes effect. Rex's reporter exists by then, and its
      # deprecation formatter takes the new stream while it has printed
      # nothing.
      def deprecation_stream=(stream)
        @deprecation_stream = stream
        reporter = Rex.world.reporter
        return unless reporter&.set_up?

        formatter = reporter.formatters.find { |each_formatter| each_formatter.is_a?(Formatters::DeprecationFormatter) }
        formatter&.retarget(reporter.deprecation_output)
      end

      def default_path=(path)
        project_source_dirs << path
        @default_path = path
      end

      def in_project_source_dir_regex
        Regexp.union(project_source_dirs.map { |directory| %r{\A#{Regexp.escape(File.expand_path(directory))}/} })
      end

      def reporter
        Rex.world.reporter
      end

      attr_reader :ordering_manager, :example_status_persistence_file_path
      attr_writer :profile_examples

      attr_reader :backtrace_formatter

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
        gem_names.each { |name| @backtrace_formatter.filter_gem(name) }
      end

      def full_backtrace?
        @backtrace_formatter.full_backtrace?
      end
      alias full_backtrace full_backtrace?

      def full_backtrace=(true_or_false)
        @backtrace_formatter.full_backtrace = true_or_false
      end

      # The number of slowest examples `--profile` lists, 10 when it is
      # given as true.
      def profile_examples
        profile = @profile_examples
        profile && !profile.is_a?(Integer) ? 10 : profile
      end
      alias profile_examples? profile_examples

      # Rex runs each `--bisect` subset in a forked process.
      def bisect_runner
        @bisect_runner || :fork
      end

      UNKNOWN_STATUS = "unknown"
      FAILED_STATUS = "failed"
      PASSED_STATUS = "passed"
      PENDING_STATUS = "pending"
      VALID_STATUSES = [UNKNOWN_STATUS, FAILED_STATUS, PASSED_STATUS, PENDING_STATUS].freeze

      def example_status_persistence_file_path=(value)
        @example_status_persistence_file_path = value
        @last_run_statuses = nil
        @spec_files_with_failures = nil
      end

      alias only_failures? only_failures

      def only_failures_but_not_configured?
        only_failures? && !example_status_persistence_file_path
      end

      # Each example id's status from the persistence file, `unknown` for
      # one the file does not list.
      def last_run_statuses
        @last_run_statuses ||= Hash.new(UNKNOWN_STATUS).tap do |statuses|
          if (path = example_status_persistence_file_path)
            ExampleStatusPersister.load_from(path).each do |example|
              status = example[:status]
              status = UNKNOWN_STATUS unless VALID_STATUSES.include?(status)
              statuses[example.fetch(:example_id)] = status
            end
          end
        end
      end

      def spec_files_with_failures
        @spec_files_with_failures ||= last_run_statuses.select { |_id, status| status == FAILED_STATUS }
                                                       .map { |id, _status| Example.parse_id(id).first }.uniq
      end

      def seed
        ordering_manager.seed
      end

      def seed=(seed)
        ordering_manager.seed = seed
      end

      def order=(type)
        ordering_manager.order = type
      end

      def seed_used?
        ordering_manager.seed_used?
      end

      def ordering_registry
        ordering_manager.ordering_registry
      end

      def register_ordering(name, strategy = nil, &block)
        if strategy
          ordering_manager.register_ordering(name, strategy)
        else
          ordering_manager.register_ordering(name, Ordering::Custom.new(block))
        end
      end

      # An option from the command line, which a later `order=` or `seed=` in
      # the configuration does not override.
      def force(hash)
        ordering_manager.force(hash)
      end

      def diff_enabled?
        @diff_enabled.nil? ? true : @diff_enabled
      end

      def diff_enabled=(value)
        @diff_enabled = value
      end

      # The most lines either side of a comparison may have for a failure to
      # print a diff of them. None by default, so every diff prints.
      attr_accessor :diff_line_limit

      # Whether a failure names the expression handed to `expect`, as in
      # `expected user.age to eq 42, got 41`. Off by default, so failures
      # read as RSpec writes them.
      attr_accessor :source_expressions

      # How many lines around the failing statement a failure prints. None
      # by default, so a failure quotes only the statement, as RSpec does.
      attr_writer :failure_context_lines

      def failure_context_lines
        @failure_context_lines || 0
      end

      # `config.add_setting :name`: a reader, a writer, and a `name?`
      # predicate, with a default and other names when given them.
      def self.add_setting(name, options = {})
        raise "Use the instance add_setting method if you want to set a default" if options.key?(:default)

        attr_writer name
        add_read_only_setting(name)
        Array(options[:alias_with]).each do |alias_name|
          alias_method alias_name, name
          alias_method "#{alias_name}=", "#{name}="
          define_predicate(alias_name)
        end
      end

      def self.add_read_only_setting(name, options = {})
        raise "Use the instance add_setting method if you want to set a default" if options.key?(:default)

        attr_reader name unless method_defined?(name)
        define_predicate(name)
      end

      def self.define_predicate(name)
        define_method("#{name}?") { !!__send__(name) }
      end

      def add_setting(name, options = {})
        default = options.delete(:default)
        (class << self; self; end).class_exec { add_setting(name, options) }
        __send__("#{name}=", default) if default
      end

      %i[default_path dry_run error_exit_code error_stream failure_exit_code fail_fast
         max_displayed_failure_line_count silence_filter_announcements
         run_all_when_everything_filtered expose_dsl_globally threadsafe
         detail_color failure_color success_color pending_color default_color fixed_color
         project_source_dirs deprecation_stream output_stream libs requires pattern
         exclude_pattern default_formatter only_failures example_status_persistence_file_path].each do |name|
        define_predicate(name) unless method_defined?("#{name}?")
      end

      add_setting :drb
      add_setting :drb_port
      add_setting :tty
      add_setting :fail_if_no_examples
      add_setting :disable_monkey_patching
      add_setting :full_cause_backtrace
      add_setting :start_time
      attr_writer :files_to_run

      # The spec files the run loads, once the runner has listed them.
      def files_to_run
        @files_to_run || []
      end

      def pending_failure_output
        @pending_failure_output || :full
      end
      alias pending_failure_output? pending_failure_output

      def pending_failure_output=(mode)
        unless %i[full no_backtrace skip].include?(mode)
          raise ArgumentError, "`pending_failure_output` can be set to :full, :no_backtrace, or :skip"
        end

        @pending_failure_output = mode
      end

      # Turns each warning Rex would print into an error.
      attr_accessor :raise_on_warning

      def bisect_runner=(value)
        @bisect_runner = value
      end

      def bisect_runner_class
        return Bisect::ForkRunner if bisect_runner == :fork

        raise "Unsupported value for `bisect_runner` (#{bisect_runner.inspect}). Rex supports only `:fork`."
      end

      def inclusion_filter=(filter)
        @inclusion_filter = Metadata.build_hash_from([filter])
      end
      alias filter inclusion_filter
      alias filter= inclusion_filter=

      def exclusion_filter=(filter)
        @exclusion_filter = Metadata.build_hash_from([filter])
      end

      def full_description=(description)
        Rex.world.add_description_filter(Regexp.union(*Array(description).map { |text| Regexp.new(text) }))
      end

      def full_description
        Rex.world.description_filter
      end

      def alias_example_to(name, *arguments)
        ExampleGroup.define_example_method(name, Metadata.build_hash_from(arguments))
      end

      def alias_example_group_to(new_name, *arguments)
        ExampleGroup.define_example_group_method(new_name, Metadata.build_hash_from(arguments))
        DSL.expose_example_group_alias(new_name)
      end

      def alias_it_behaves_like_to(new_name, report_label = "")
        ExampleGroup.define_nested_shared_group_method(new_name, report_label)
      end
      alias alias_it_should_behave_like_to alias_it_behaves_like_to

      # Every example can read itself through `method_name`.
      def expose_current_running_example_as(method_name)
        exposing = Module.new { define_method(method_name) { Rex.current_example } }
        include(exposing)
      end

      # Runs the block once, when the first example matching `filters` is
      # defined.
      def when_first_matching_example_defined(*filters)
        specified = Metadata.build_hash_from(filters)
        entry = nil
        callback = lambda do |metadata|
          next unless metadata.key?(:example_group)

          @derived_metadata_blocks.delete(entry)
          yield
        end
        entry = [specified, callback]
        (@derived_metadata_blocks ||= []) << entry
      end

      DEFAULT_DOCSTRING_FORMATTER = ->(text) { text }

      def format_docstrings(&block)
        @format_docstrings_block = block || DEFAULT_DOCSTRING_FORMATTER
      end

      def format_docstrings_block
        @format_docstrings_block || DEFAULT_DOCSTRING_FORMATTER
      end

      # Whether `RSpec` names Rex. It does from load, and a suite with an
      # `RSpec` of its own sets this to false to take the name back.
      def rspec_constant
        defined?(::RSpec) && ::RSpec.equal?(Rex) ? true : false
      end
      alias rspec_constant? rspec_constant

      def rspec_constant=(value)
        if value
          Object.const_set(:RSpec, Rex) unless defined?(::RSpec)
        elsif rspec_constant
          Object.__send__(:remove_const, :RSpec)
        end
      end

      def on_example_group_definition(&block)
        on_example_group_definition_callbacks << block
      end

      def on_example_group_definition_callbacks
        @on_example_group_definition_callbacks ||= []
      end
    end

    # The object `config.expect_with(:rspec) { |c| }` yields.
    class ExpectationConfiguration
      attr_accessor :include_chain_clauses_in_custom_matcher_descriptions,
                    :max_formatted_output_length, :strict_predicate_matchers

      FALSE_POSITIVE_BEHAVIOURS = {
        warn: ->(message) { Rex.warning(message) },
        raise: ->(message) { raise ArgumentError, message },
        nothing: ->(_message) { true }
      }.freeze

      def on_potential_false_positives
        @on_potential_false_positives || :warn
      end

      def on_potential_false_positives=(behavior)
        unless FALSE_POSITIVE_BEHAVIOURS.key?(behavior)
          raise ArgumentError, "Supported values are: #{FALSE_POSITIVE_BEHAVIOURS.keys}"
        end

        @on_potential_false_positives = behavior
      end

      def false_positives_handler
        FALSE_POSITIVE_BEHAVIOURS.fetch(on_potential_false_positives)
      end

      def initialize(configuration)
        @configuration = configuration
      end

      def syntax=(syntaxes)
        @configuration.expectation_syntax = syntaxes
      end

      def syntax
        @configuration.expectation_syntax
      end
    end

    # The object `config.mock_with(:rspec) { |c| }` yields, which is also the
    # mocks configuration itself.
    class MockConfiguration
      attr_accessor :verify_doubled_constant_names, :allow_message_expectations_on_nil,
                    :transfer_nested_constants, :temporarily_suppress_partial_double_verification,
                    :color

      attr_writer :verify_partial_doubles

      def initialize(configuration)
        @configuration = configuration
        @verify_doubled_constant_names = false
        @temporarily_suppress_partial_double_verification = false
      end

      # On unless a suite turns it off, so a stub of a method the object
      # does not have fails where it is written.
      def verify_partial_doubles?
        @verify_partial_doubles.nil? ? true : @verify_partial_doubles
      end
      alias verify_partial_doubles verify_partial_doubles?

      def syntax=(syntaxes)
        syntaxes = Array(syntaxes)
        Rex::Mocks::Syntax.enable_should if syntaxes.include?(:should)
        @syntax = syntaxes
      end

      def syntax
        @syntax || [:expect]
      end

      def before_verifying_doubles(&block)
        (@verifying_double_callbacks ||= []) << block
      end
      alias when_declaring_verified_double before_verifying_doubles

      def verifying_double_callbacks
        @verifying_double_callbacks || []
      end
    end
  end

  def self.configuration
    @configuration ||= Core::Configuration.new
  end

  def self.configure
    yield configuration
  end

  def self.warning(text, options = {})
    warn_with("WARNING: #{text}.", options)
  end

  # Prints a warning naming where it was called from, or raises it when
  # `config.raise_on_warning` is on.
  def self.warn_with(message, options = {})
    if options[:use_spec_location_as_call_site]
      message += "." unless message.end_with?(".")
      message += " Warning generated from spec at `#{current_example.location}`." if current_example
    end
    call_site = options.fetch(:call_site) { Core.first_non_rex_line }
    message += " Use #{options[:replacement]} instead." if options[:replacement]
    message += " Called from #{call_site}." if call_site
    raise message if configuration.raise_on_warning

    ::Kernel.warn(message)
  end

  def self.reset
    @configuration = nil
    @world = nil
  end
end
