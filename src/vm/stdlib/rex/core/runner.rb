require "fileutils"
require "shellwords"

module Rex
  module Core
    # The options a run takes, read from the option files, the command line,
    # and SPEC_OPTS, in that order, so a later source overrides an earlier one.
    class ConfigurationOptions
      attr_reader :options, :args

      def initialize(args)
        @args = args.dup
        @options = {}
      end

      def parse
        sources = []
        command_line = Parser.new.parse(@args.dup)
        if command_line[:custom_options_file]
          sources << options_from_file(command_line[:custom_options_file])
        else
          sources << options_from_file(File.join(File.expand_path("~"), ".rspec")) if ENV["HOME"]
          sources << options_from_file(".rspec")
          sources << options_from_file(".rex")
          sources << options_from_file(".rspec-local")
          sources << options_from_file(".rex-local")
        end
        sources << command_line
        sources << Parser.new.parse(Shellwords.split(ENV["SPEC_OPTS"])) if ENV["SPEC_OPTS"]
        @options = merge(sources)
      end

      private

      def options_from_file(path)
        return {} unless File.exist?(path)

        words = File.readlines(path).map { |line| line.sub(/#.*/, "").strip }.reject(&:empty?)
        Parser.new.parse(words.flat_map { |line| Shellwords.split(line) })
      end

      # A formatter named on a later source replaces the earlier ones, while
      # requires, load paths, and filters accumulate.
      APPENDING_KEYS = %i[requires libs inclusion_filter exclusion_filter full_description files_or_directories_to_run].freeze

      def merge(sources)
        merged = {}
        sources.each do |source|
          source.each do |key, value|
            if APPENDING_KEYS.include?(key) && merged[key]
              merged[key] = merged[key].is_a?(Hash) ? merged[key].merge(value) : merged[key] + value
            else
              merged[key] = value
            end
          end
        end
        merged
      end
    end

    # Reads the words of a command line into options.
    class Parser
      def parse(words)
        options = {}
        until words.empty?
          word = words.shift
          name, attached = word.start_with?("--") ? word.split("=", 2) : [word, nil]
          take = lambda do
            attached || words.shift || raise(OptionError, "missing argument: #{name}")
          end
          case name
          when "-I"
            (options[:libs] ||= []).concat(take.call.split(File::PATH_SEPARATOR))
          when /\A-I(.+)/
            (options[:libs] ||= []) << Regexp.last_match(1)
          when "-r", "--require"
            (options[:requires] ||= []) << take.call
          when /\A-r(.+)/
            (options[:requires] ||= []) << Regexp.last_match(1)
          when "-O", "--options"
            options[:custom_options_file] = take.call
          when "--order"
            options[:order] = take.call
          when "--seed"
            options[:order] = "rand:#{take.call}"
          when "--bisect"
            options[:bisect] = attached || true
          when "--fail-fast"
            options[:fail_fast] = attached ? attached.to_i : 1
          when "--no-fail-fast"
            options[:fail_fast] = false
          when "--jobs"
            options[:jobs] = Parser.jobs_from(take.call)
          when "--verify-parallel"
            options[:verify_parallel] = true
          when "--coverage"
            options[:coverage] = true
          when "--coverage-format"
            format = take.call
            unless Rex::Core::CoverageReport::FORMATS.include?(format)
              raise OptionError, "invalid argument: --coverage-format #{format}"
            end

            (options[:coverage_formats] ||= []) << format
          when "--coverage-dir"
            options[:coverage_dir] = take.call
          when "--coverage-threshold"
            threshold = Float(take.call, exception: false)
            raise OptionError, "invalid argument: --coverage-threshold #{attached}" unless threshold

            options[:coverage_threshold] = threshold
          when "--coverage-exclude"
            (options[:coverage_exclude] ||= []) << take.call
          when "--failure-exit-code"
            options[:failure_exit_code] = take.call.to_i
          when "--error-exit-code"
            options[:error_exit_code] = take.call.to_i
          when "-X", "--drb", "--no-drb", "--drb-port"
            words.shift if name == "--drb-port" && !attached
          when "--dry-run"
            options[:dry_run] = true
          when "-f", "--format", "--formatter"
            (options[:formatters] ||= []) << [take.call]
          when /\A-f(.+)/
            (options[:formatters] ||= []) << [Regexp.last_match(1)]
          when "-o", "--out"
            raise OptionError, "--out must come after a --format" if (options[:formatters] ||= []).empty? && false
            path = take.call
            if options[:formatters].nil? || options[:formatters].empty?
              options[:formatters] = [["progress", path]]
            else
              options[:formatters].last[1] = path
            end
          when "--deprecation-out"
            options[:deprecation_stream] = take.call
          when "-b", "--backtrace"
            options[:full_backtrace] = true
          when "-c", "--color", "--colour"
            options[:color_mode] = :automatic
          when "--force-color", "--force-colour"
            raise OptionAbort, "Please only use one of `--force-color` and `--no-color`" if options[:color_mode] == :off

            options[:color_mode] = :on
          when "--diff"
            options[:diff] = true
          when "--no-diff"
            options[:diff] = false
          when "--no-color", "--no-colour"
            raise OptionAbort, "Please only use one of --force-color and --no-color" if options[:color_mode] == :on

            options[:color_mode] = :off
          when "-p", "--profile"
            options[:profile_examples] = if attached
                                           attached.to_i
                                         elsif words.first =~ /\A\d+\z/
                                           words.shift.to_i
                                         else
                                           true
                                         end
          when "--no-profile"
            options[:profile_examples] = false
          when "-w", "--warnings"
            options[:warnings] = true
          when "-P", "--pattern"
            options[:pattern] = take.call
          when "--exclude-pattern"
            options[:exclude_pattern] = take.call
          when "-e", "--example"
            (options[:full_description] ||= []) << Regexp.new(Regexp.escape(take.call))
          when "-E", "--example-matches"
            (options[:full_description] ||= []) << Regexp.new(take.call)
          when "-t", "--tag"
            tag = take.call
            filter_type = tag.start_with?("~") ? :exclusion_filter : :inclusion_filter
            name_and_value = tag.sub(/\A~/, "").sub(/\A:/, "")
            key, value = name_and_value.split(":", 2)
            value = if value.nil?
                      true
                    elsif value =~ /\A\d+\z/
                      value.to_i
                    elsif value.start_with?(":")
                      value[1..].to_sym
                    else
                      value
                    end
            (options[filter_type] ||= {})[key.to_sym] = value
          when "--default-path"
            options[:default_path] = take.call
          when "--only-failures"
            configure_only_failures(options)
          when "-n", "--next-failure"
            configure_only_failures(options)
            options[:fail_fast] = 1
            options[:order] ||= "defined"
          when "--init"
            options[:runner] = :init
          when "-v", "--version"
            options[:runner] = :version
          when "-h", "--help"
            options[:runner] = :help
          when "--"
            (options[:files_or_directories_to_run] ||= []).concat(words)
            words = []
          else
            raise OptionError, "invalid option: #{word}" if word.start_with?("-") && word != "-"

            (options[:files_or_directories_to_run] ||= []) << word
          end
        end
        options
      end

      # A number of workers, or `auto` for one per processor.
      def self.jobs_from(text)
        if text == "auto"
          require "etc"
          return Etc.nprocessors
        end

        jobs = Integer(text, exception: false)
        raise OptionError, "invalid argument: --jobs #{text}" unless jobs && jobs.positive?

        jobs
      end

      def configure_only_failures(options)
        options[:only_failures] = true
        (options[:inclusion_filter] ||= {})[:last_run_status] = "failed"
      end
    end

    class OptionError < StandardError
    end

    # Options that contradict each other, which end the run with only the
    # message.
    class OptionAbort < StandardError
    end

    # `metorex spec`: reads the options, loads the spec files, runs them, and
    # answers the exit status.
    class Runner
      HELP = <<~TEXT
        Usage: metorex spec [options] [files or directories]

            -I PATH                            Specify PATH to add to $LOAD_PATH (may be used more than once).
            -r, --require PATH                 Require a file.
            -O, --options PATH                 Specify the path to a custom options file.
                --order TYPE[:SEED]            Run examples by the specified order type.
                                                 [defined] examples and groups are run in the order they are defined
                                                 [rand]    randomize the order of groups and examples
                                                 [random]  alias for rand
                                                 [random:SEED] e.g. --order random:123
                --seed SEED                    Equivalent of --order rand:SEED.
                --bisect[=verbose]             Repeatedly runs the suite in order to isolate the failures to the
                                                 smallest reproducible case.
                --[no-]fail-fast[=COUNT]       Abort the run after a certain number of failures (1 by default).
                --failure-exit-code CODE       Override the exit code used when there are failing specs.
                --error-exit-code CODE         Override the exit code used when there are errors loading or running specs outside of examples.
                --dry-run                      Print the formatter output of your suite without
                                                 running any examples or hooks

          **** Output ****

            -f, --format FORMATTER             Choose a formatter.
                                                 [p]rogress (default - dots)
                                                 [d]ocumentation (group and example names)
                                                 [h]tml
                                                 [j]son
                                                 custom formatter class name
            -o, --out FILE                     Write output to a file instead of $stdout. This option applies
                                                 to the previously specified --format, or the default format
                                                 if no format is specified.
                --deprecation-out FILE         Write deprecation warnings to a file instead of $stderr.
            -b, --backtrace                    Enable full backtrace.
            -c, --color, --colour              Enable color in the output.
                --force-color, --force-colour  Force the output to be in color, even if the output is not a TTY
                --no-color, --no-colour        Force the output to not be in color, even if the output is a TTY
                --[no-]diff                    Print a diff with failures that compare two values (default: on).
                --jobs N                       Run the top-level groups in N worker processes, or one per processor
                                                 with `auto` (default: 1).
                --verify-parallel              After a parallel run, run the suite again serially with the same seed
                                                 and warn about examples whose outcome differs.
                --coverage                     Report which lines and branches of the loaded files ran, leaving out
                                                 the spec files and the spec directory.
                --coverage-format FORMAT       summary (the default), lcov, or json; may be given more than once.
                --coverage-dir DIR             Where the lcov and json files are written (default: coverage).
                --coverage-threshold PERCENT   Fail the run when line coverage is below PERCENT.
                --coverage-exclude PATTERN     Leave out files matching the glob PATTERN; may be given more than once.
            -p, --[no-]profile [COUNT]         Enable profiling of examples and list the slowest examples (default: 10).
            -w, --warnings                     Enable ruby warnings

          **** Filtering/tags ****

            -P, --pattern PATTERN              Load files matching pattern (default: "spec/**/*_spec.rb").
                --exclude-pattern PATTERN      Load files except those matching pattern. Opposite effect of --pattern.
            -e, --example STRING               Run examples whose full nested names include STRING (may be
                                                 used more than once)
            -E, --example-matches REGEX        Run examples whose full nested names match REGEX (may be
                                                 used more than once)
            -t, --tag TAG[:VALUE]              Run examples with the specified tag, or exclude examples
                                               by adding ~ before the tag.
                                                 - e.g. ~slow
                                                 - TAG is always converted to a symbol
                --default-path PATH            Set the default path where RSpec looks for examples (can
                                                 be a path to a file or a directory).
                --only-failures                Filter to just the examples that failed the last time they ran.
            -n, --next-failure                 Apply `--only-failures` and abort after one failure.

          **** Utility ****

                --init                         Initialize your project with Rex.
            -v, --version                      Display the version.
            -h, --help                         You're looking at it.
      TEXT

      def self.run(args, err = $stderr, out = $stdout)
        trap_interrupt
        new(args, err, out).run
      end

      # The first interrupt lets the running example finish and prints the
      # summary for what ran. A second one ends the process at once.
      def self.trap_interrupt
        trap("INT") { handle_interrupt }
      end

      def self.handle_interrupt
        if Rex.world.wants_to_quit
          exit!(1)
        else
          Rex.world.wants_to_quit = true
          $stderr.puts(
            "\nRSpec is shutting down and will print the summary report... Interrupt again to force quit " \
            "(warning: at_exit hooks will be skipped if you force quit)."
          )
        end
      end

      def initialize(args, err, out)
        @args = args
        @err = err
        @out = out
      end

      def run
        started = Rex::Core.monotonic_now
        options = ConfigurationOptions.new(@args)
        begin
          options = options.parse
        rescue OptionError => error
          @err.puts error.message
          @err.puts "Please use --help for a listing of valid options"
          return 1
        rescue OptionAbort => error
          @err.puts error.message
          return 1
        end

        case options[:runner]
        when :help
          @out.puts HELP
          return 0
        when :version
          @out.puts "Rex #{Rex::VERSION} (RSpec #{Rex::RSPEC_VERSION} compatible)"
          return 0
        when :init
          return init_project
        end

        configuration = Rex.configuration
        configuration.output_stream = @out
        configuration.error_stream = @err
        apply(options, configuration)
        Rex::Core::DSL.expose_globally! if configuration.expose_dsl_globally
        return bisect(options, configuration) if options[:bisect]

        files = spec_files(options, configuration)
        configuration.files_to_run = files
        # Measuring starts before anything the suite requires is loaded,
        # since only a file loaded while it is on is counted.
        CoverageReport.start if configuration.coverage
        configuration.requires.each { |path| require_handling_errors(path) }
        load_spec_helper(configuration)

        world = Rex.world
        set_up_reporter(configuration, world)

        files.each { |file| load_spec_file(file, world) }
        load_time = Rex::Core.monotonic_now - started

        if configuration.only_failures_but_not_configured?
          world.reporter.message("\nTo use `--only-failures`, you must first set `config.example_status_persistence_file_path`.")
          world.reporter.close
          return 1
        end

        world.announce_filters
        if world.non_example_failure
          world.reporter.start(0, load_time)
          world.reporter.finish
          return exit_code_for_errors(configuration)
        end

        groups = world.ordered_example_groups
        success = true
        example_count = world.example_count(groups)
        no_examples_failed = example_count.zero? && configuration.fail_if_no_examples
        world.reporter.report(example_count, load_time) do |reporter|
          next if no_examples_failed

          Rex::Core::Suite.run_hooks(:before)
          success = if parallel?(configuration, groups)
                      !Parallel.run(groups, configuration.jobs, reporter) && !world.wants_to_quit
                    else
                      groups.map { |group| group.run(reporter) }.all?
                    end
          Rex::Core::Suite.run_hooks(:after)
        end
        return configuration.failure_exit_code if no_examples_failed

        Parallel.verify(groups, world) if parallel?(configuration, groups) && configuration.verify_parallel
        coverage_met = configuration.coverage ? report_coverage(configuration, world) : true
        Rex::Core::ExampleStatusPersister.persist(world.all_examples, configuration.example_status_persistence_file_path) if configuration.example_status_persistence_file_path && !configuration.dry_run

        if world.non_example_failure
          exit_code_for_errors(configuration)
        elsif success && world.reporter.failed_examples.empty? && coverage_met
          0
        else
          configuration.failure_exit_code
        end
      end

      # The counts of this process and of each worker, reported together.
      def report_coverage(configuration, world)
        data = CoverageReport.collect
        world.worker_coverage.each { |counts| CoverageReport.merge(data, counts) }
        CoverageReport.report(data, configuration)
      end

      # Runs the examples `ids` name with `formatter` as the only one
      # listening, in the process `--bisect` forked for it, and answers what
      # the formatter collected.
      def run_bisect_subset(ids, formatter, output)
        configuration = Rex.configuration
        configuration.output_stream = output
        configuration.error_stream = output
        configuration.files_or_directories_to_run = ids
        world = Rex.world
        world.reporter = Reporter.new(configuration)
        world.reporter.add_formatter(formatter)
        spec_files(@options, configuration).each { |file| load_spec_file(file, world) }
        world.announce_filters
        groups = world.ordered_example_groups
        world.reporter.report(world.example_count(groups), 0) do |reporter|
          Rex::Core::Suite.run_hooks(:before)
          groups.each { |group| group.run(reporter) }
          Rex::Core::Suite.run_hooks(:after)
        end
        formatter.results
      end

      private

      # `--bisect` loads the requires and spec_helper here, and each spec
      # file only in the processes it forks.
      def bisect(options, configuration)
        require "rex/core/bisect"
        @options = options
        configuration.requires.each { |path| require_handling_errors(path) }
        load_spec_helper(configuration)
        formatter_class = options[:bisect] == "verbose" ? Formatters::BisectDebugFormatter : Formatters::BisectProgressFormatter
        formatter = formatter_class.new(@out, configuration.bisect_runner)
        Bisect::Coordinator.bisect_with(self, @args, formatter) ? 0 : configuration.failure_exit_code
      end

      # Workers run the groups when there are jobs to share them between and
      # the run is a real one.
      def parallel?(configuration, groups)
        configuration.jobs > 1 && groups.size > 1 && !configuration.dry_run
      end

      def exit_code_for_errors(configuration)
        configuration.error_exit_code || configuration.failure_exit_code
      end

      def apply(options, configuration)
        configuration.libs = options[:libs] || []
        configuration.libs.reverse_each { |lib| $LOAD_PATH.unshift(File.expand_path(lib)) }
        configuration.default_path = options[:default_path] if options[:default_path]
        %w[lib spec].reverse_each do |directory|
          path = File.expand_path(directory == "spec" ? configuration.default_path : directory)
          $LOAD_PATH.unshift(path) if File.directory?(path) && !$LOAD_PATH.include?(path)
        end
        configuration.requires = options[:requires] || []
        configuration.pattern = options[:pattern] if options[:pattern]
        configuration.exclude_pattern = options[:exclude_pattern] if options[:exclude_pattern]
        configuration.only_failures = true if options[:only_failures]
        configuration.fail_fast = options[:fail_fast] if options.key?(:fail_fast)
        configuration.failure_exit_code = options[:failure_exit_code] if options[:failure_exit_code]
        configuration.error_exit_code = options[:error_exit_code] if options[:error_exit_code]
        configuration.dry_run = true if options[:dry_run]
        configuration.full_backtrace = true if options[:full_backtrace]
        configuration.color_mode = options[:color_mode] if options[:color_mode]
        configuration.diff_enabled = options[:diff] if options.key?(:diff)
        configuration.jobs = options[:jobs] if options[:jobs]
        configuration.verify_parallel = true if options[:verify_parallel]
        configuration.coverage = true if options[:coverage]
        configuration.coverage_formats = options[:coverage_formats] if options[:coverage_formats]
        configuration.coverage_dir = options[:coverage_dir] if options[:coverage_dir]
        configuration.coverage_threshold = options[:coverage_threshold] if options[:coverage_threshold]
        configuration.coverage_exclude.concat(options[:coverage_exclude]) if options[:coverage_exclude]
        configuration.deprecation_stream = options[:deprecation_stream] if options[:deprecation_stream]
        configuration.profile_examples = options[:profile_examples] if options.key?(:profile_examples)
        configuration.warnings = true if options[:warnings]
        $VERBOSE = true if options[:warnings]
        configuration.force(order: options[:order]) if options[:order]
        configuration.inclusion_filter.update(options[:inclusion_filter]) if options[:inclusion_filter]
        configuration.exclusion_filter.update(options[:exclusion_filter]) if options[:exclusion_filter]
        Rex.world.add_description_filter(Regexp.union(*options[:full_description])) if options[:full_description]
        (options[:formatters] || []).each { |name, output| configuration.add_formatter(name, output) }
        configuration.files_or_directories_to_run = options[:files_or_directories_to_run] || []
      end

      def spec_files(options, configuration)
        paths = configuration.files_or_directories_to_run
        paths = [configuration.default_path] if paths.empty?
        files = []
        paths.each do |path|
          path, filter = split_location(path)
          if File.directory?(path)
            files.concat(files_in_directory(path, configuration))
          elsif File.exist?(path)
            files << path
          elsif paths.size == 1 && path == configuration.default_path
            next
          else
            @err.puts "Rex could not find a file or directory at #{path}."
            next
          end
          Rex.world.add_location_filter(path, filter) if filter
        end
        files = files.map { |file| File.expand_path(file) }.uniq
        return files unless configuration.only_failures

        # Only the files where an example failed last time, unless none of
        # the files given has one.
        with_failures = configuration.spec_files_with_failures
        failing = files.select { |file| with_failures.include?(Rex::Core.relative_path(file)) }
        failing.empty? ? files : failing
      end

      def split_location(path)
        if (match = path.match(/\A(.+?)((?::\d+)+)\z/))
          [match[1], match[2].split(":").reject(&:empty?).map(&:to_i)]
        elsif (match = path.match(/\A(.+?)\[([\d:,]+)\]\z/))
          [match[1], nil].tap { Rex.world.add_id_filter(match[1], match[2].split(",")) }
        else
          [path, nil]
        end
      end

      def files_in_directory(directory, configuration)
        matched = Dir[file_glob_from(directory, configuration.pattern)].map { |file| File.expand_path(file) }
        unless configuration.exclude_pattern.to_s.empty?
          excluded = Dir[file_glob_from(directory, configuration.exclude_pattern)].map { |file| File.expand_path(file) }
          matched -= excluded
        end
        matched.sort
      end

      # A pattern written from the project root is used as it is, and one
      # written inside the directory is joined to it.
      def file_glob_from(directory, pattern)
        stripped = "{#{pattern.gsub(/\s*,\s*/, ',')}}"
        return stripped if pattern =~ %r{\A(\./)?#{Regexp.escape(directory)}} || pattern.start_with?("/")

        File.join(directory, stripped)
      end

      # Rex loads spec_helper.rb first whenever it is there, unless a
      # `--require` already names it.
      def load_spec_helper(configuration)
        helper = File.expand_path(File.join(configuration.default_path, "spec_helper.rb"))
        return unless File.exist?(helper)
        return if configuration.requires.any? { |path| required_file(path, configuration) == helper }

        require_handling_errors(helper)
      end

      def required_file(path, configuration)
        [path, "#{path}.rb"].map { |name| File.expand_path(name, configuration.default_path) }
                            .find { |candidate| File.file?(candidate) }
      end

      def require_handling_errors(path)
        require path
      rescue SyntaxError, StandardError, ScriptError => error
        world = Rex.world
        world.reporter ||= Reporter.new(Rex.configuration)
        world.reporter.notify_non_example_exception(error, "An error occurred while loading #{path}.")
      end

      def load_spec_file(file, world)
        Rex.configuration.loaded_spec_files << file
        load file
      rescue SyntaxError, StandardError, ScriptError => error
        world.reporter ||= Reporter.new(Rex.configuration)
        world.reporter.notify_non_example_exception(error, "An error occurred while loading #{Rex::Core.relative_path(file)}.")
      end

      def set_up_reporter(configuration, world)
        reporter = world.reporter || Reporter.new(configuration)
        formatters = configuration.formatters
        formatters = [[configuration.default_formatter, nil]] if formatters.empty?
        formatters.each do |name, output|
          reporter.add_formatter(Formatters.build(name, output, configuration))
        end
        reporter.setup_default(configuration.output_stream)
        world.reporter = reporter
      end

      def init_project
        written = []
        unless File.exist?(".rspec")
          File.write(".rspec", "--require spec_helper\n")
          written << ".rspec"
        end
        Dir.mkdir("spec") unless File.directory?("spec")
        unless File.exist?("spec/spec_helper.rb")
          File.write("spec/spec_helper.rb", <<~HELPER)
            Rex.configure do |config|
              config.expect_with :rspec do |expectations|
                expectations.include_chain_clauses_in_custom_matcher_descriptions = true
              end

              config.mock_with :rspec do |mocks|
                mocks.verify_partial_doubles = true
              end

              config.shared_context_metadata_behavior = :apply_to_host_groups
            end
          HELPER
          written << "spec/spec_helper.rb"
        end
        written.each { |path| @out.puts "  create   #{path}" }
        0
      end
    end

    # `before(:suite)` and `after(:suite)`.
    module Suite
      def self.run_hooks(kind)
        return if Rex.configuration.dry_run

        Rex.configuration.hooks.for(kind, :suite).each do |hook|
          begin
            hook.block.call
          rescue *Rex::Core::DANGEROUS_EXCEPTIONS
            raise
          rescue Exception => error
            Rex.world.reporter.notify_non_example_exception(error, "An error occurred in a `#{kind}(:suite)` hook.")
          end
        end
      end
    end

    # The file `example_status_persistence_file_path` names, recording each
    # example's last status for `--only-failures` and `--next-failure`.
    class ExampleStatusPersister
      def self.load_from(file_name)
        return [] unless File.exist?(file_name)

        ExampleStatusParser.parse(File.read(file_name))
      end

      def self.persist(examples, file_name)
        new(examples, file_name).persist
      end

      def initialize(examples, file_name)
        @examples = examples
        @file_name = file_name
      end

      def persist
        directory = File.dirname(@file_name)
        FileUtils.mkdir_p(directory) unless File.directory?(directory)
        previous = File.exist?(@file_name) ? File.read(@file_name) : ""
        File.write(@file_name, dump_statuses(previous).to_s)
      end

      private

      def dump_statuses(unparsed_previous_runs)
        statuses_from_previous_runs = ExampleStatusParser.parse(unparsed_previous_runs)
        merged_statuses = ExampleStatusMerger.merge(statuses_from_this_run, statuses_from_previous_runs)
        ExampleStatusDumper.dump(merged_statuses)
      end

      def statuses_from_this_run
        @examples.map do |example|
          result = example.execution_result
          {
            example_id: example.id,
            status: result.status ? result.status.to_s : Configuration::UNKNOWN_STATUS,
            run_time: result.run_time ? Formatters.format_duration(result.run_time) : ""
          }
        end
      end
    end

    # This run's statuses over the previous runs'. An example this run loaded
    # and did not run keeps its last status, and one whose file was loaded,
    # or no longer exists, without it is dropped.
    class ExampleStatusMerger
      def self.merge(this_run, from_previous_runs)
        new(this_run, from_previous_runs).merge
      end

      def initialize(this_run, from_previous_runs)
        @this_run = hash_from(this_run)
        @from_previous_runs = hash_from(from_previous_runs)
        @file_exists_cache = Hash.new { |hash, file| hash[file] = File.exist?(file) }
      end

      def merge
        delete_previous_examples_that_no_longer_exist
        @this_run.merge(@from_previous_runs) do |_id, new, old|
          new.fetch(:status) == Configuration::UNKNOWN_STATUS ? old : new
        end.values.sort_by { |example| sort_value_from(example) }
      end

      private

      def hash_from(example_list)
        example_list.each_with_object({}) do |example, hash|
          hash[example.fetch(:example_id)] = example
        end
      end

      def delete_previous_examples_that_no_longer_exist
        @from_previous_runs.delete_if { |id, _| example_must_no_longer_exist?(id) }
      end

      def example_must_no_longer_exist?(id)
        return false if @this_run.key?(id)

        spec_file = spec_file_from(id)
        return true if loaded_spec_files.include?(spec_file)

        !@file_exists_cache[spec_file]
      end

      def loaded_spec_files
        @loaded_spec_files ||= @this_run.keys.map { |id| spec_file_from(id) }.uniq
      end

      def spec_file_from(id)
        id.split("[").first
      end

      def sort_value_from(example)
        file, scoped_id = Example.parse_id(example.fetch(:example_id))
        [file, *scoped_id.split(":").map { |part| Integer(part) }]
      end
    end

    # The statuses as a table of ` | `-separated columns, one row each.
    class ExampleStatusDumper
      def self.dump(examples)
        new(examples).dump
      end

      def initialize(examples)
        @examples = examples
      end

      def dump
        return nil if @examples.empty?

        (formatted_header_rows + formatted_value_rows).join("\n") << "\n"
      end

      private

      def formatted_header_rows
        dividers = column_widths.map { |width| "-" * width }
        [formatted_row_from(headers.map(&:to_s)), formatted_row_from(dividers)]
      end

      def formatted_value_rows
        rows.map { |row| formatted_row_from(row) }
      end

      def rows
        @rows ||= @examples.map { |example| example.values_at(*headers) }
      end

      def formatted_row_from(row_values)
        padded_values = row_values.each_with_index.map do |value, index|
          value.ljust(column_widths[index])
        end
        padded_values.join(" | ") << " |"
      end

      def headers
        @headers ||= @examples.first.keys
      end

      def column_widths
        @column_widths ||= headers.each_with_index.map do |header, index|
          (rows.map { |row| row[index] } + [header.to_s]).map(&:length).max
        end
      end
    end

    # Reads the table ExampleStatusDumper writes.
    class ExampleStatusParser
      def self.parse(string)
        new(string).parse
      end

      def initialize(string)
        @header_line, _, *@row_lines = string.lines.to_a
      end

      def parse
        @row_lines.map { |line| parse_row(line) }
      end

      private

      def parse_row(line)
        headers.zip(split_line(line)).to_h
      end

      def headers
        @headers ||= split_line(@header_line).grep(/\S/).map(&:to_sym)
      end

      def split_line(line)
        line.split(/\s+\|\s+?/, -1)
      end
    end
  end

  VERSION = "0.1.0"
  RSPEC_VERSION = "3.13"
end
