require "stringio"

module Rex
  module Core
    # `--bisect`: runs subsets of a failing suite, each in a forked process,
    # until it finds the fewest non-failing examples the failures need to run
    # first.
    module Bisect
      ExampleSetDescriptor = Struct.new(:all_example_ids, :failed_example_ids)

      class BisectFailedError < StandardError
        def self.for_failed_spec_run(spec_output)
          new("Failed to get results from the spec run. Spec run output:\n\n#{spec_output}")
        end
      end

      # Hands an event and its values to the formatter, as a notification
      # with one reader per value.
      class Notifier
        def initialize(formatter)
          @formatter = formatter
        end

        def publish(event, values = {})
          return unless @formatter.respond_to?(event)

          notification = if values.empty?
                           Formatters::NullNotification.new(nil)
                         else
                           Struct.new(*values.keys).new(*values.values)
                         end
          @formatter.__send__(event, notification)
        end
      end

      # A pipe carrying one result from a forked run back to the parent.
      class Channel
        def initialize
          @read_io, @write_io = IO.pipe
          @write_io.binmode
          @read_io.binmode
        end

        def send(message)
          packet = Marshal.dump(message)
          @write_io.write("#{packet.bytesize}\n#{packet}")
          @write_io.flush
        end

        def receive
          packet_size = Integer(@read_io.gets)
          Marshal.load(@read_io.read(packet_size))
        end

        def close
          @read_io.close
          @write_io.close
        end
      end

      # The command line a subset runs with, and the one the report prints.
      class ShellCommand
        SHELLS_ALLOWING_UNQUOTED_IDS = %w[bash ksh fish].freeze

        attr_reader :original_cli_args

        def initialize(original_cli_args)
          @original_cli_args = original_cli_args.reject { |arg| arg.start_with?("--bisect") }
        end

        def repro_command_from(locations)
          parts = environment_repro_parts
          parts << "rspec"
          parts.concat(organize_ids(locations))
          parts.concat(original_cli_args_without_locations)
          parts.join(" ")
        end

        def original_locations
          parsed_original_cli_options[:files_or_directories_to_run] || []
        end

        private

        def environment_repro_parts
          return [] unless ENV.key?("SPEC_OPTS")

          spec_opts = Shellwords.join(Shellwords.split(ENV.fetch("SPEC_OPTS", "")).reject { |arg| arg =~ /^--bisect/ })
          [%(SPEC_OPTS="#{spec_opts}")]
        end

        def original_cli_args_without_locations
          @original_cli_args - original_locations
        end

        def parsed_original_cli_options
          @parsed_original_cli_options ||= Parser.new.parse(@original_cli_args.dup)
        end

        # Each file's ids joined into one `file[id,id]` argument, files in
        # order and ids in definition order.
        def organize_ids(ids)
          grouped = Hash.new { |hash, file| hash[file] = [] }
          ids.each do |id|
            file, scoped_id = Example.parse_id(id)
            grouped[file] << scoped_id
          end
          grouped.sort_by(&:first).map do |file, scoped_ids|
            scoped_ids = scoped_ids.sort_by { |scoped_id| scoped_id.split(":").map(&:to_i) }
            conditionally_quote("#{file}[#{scoped_ids.join(',')}]")
          end
        end

        def conditionally_quote(id)
          return id if SHELLS_ALLOWING_UNQUOTED_IDS.include?(ENV["SHELL"].to_s.split("/").last)

          "'#{id.gsub("'", "\\\\'")}'"
        end
      end

      # Collects which examples ran and which failed, and stops the run once
      # every expected failure has failed.
      class CaptureFormatter
        Formatters.register self, :example_failed, :example_finished, :start_dump

        attr_reader :results

        def initialize(expected_failures)
          @all_example_ids = []
          @failed_example_ids = []
          @remaining_failures = expected_failures.dup
        end

        def example_failed(notification)
          @failed_example_ids << notification.example.id
        end

        def example_finished(notification)
          @all_example_ids << notification.example.id
          return unless @remaining_failures.include?(notification.example.id)

          @remaining_failures.delete(notification.example.id)
          status = notification.example.execution_result.status
          return if status == :failed && !@remaining_failures.empty?

          Rex.world.wants_to_quit = true
        end

        def start_dump(_notification)
          @results = ExampleSetDescriptor.new(@all_example_ids, @failed_example_ids)
        end
      end

      # Runs each subset in a forked process, so a spec file is never loaded
      # in the parent and one run's state cannot leak into the next.
      class ForkRunner
        def self.start(shell_command, spec_runner)
          instance = new(shell_command, spec_runner)
          yield instance
        ensure
          instance&.shutdown
        end

        def self.name
          :fork
        end

        def initialize(shell_command, spec_runner)
          @shell_command = shell_command
          @spec_runner = spec_runner
          @channel = Channel.new
        end

        def run(locations)
          dispatch_run(ExampleSetDescriptor.new(locations, original_results.failed_example_ids))
        end

        def original_results
          @original_results ||= dispatch_run(ExampleSetDescriptor.new(@shell_command.original_locations, []))
        end

        def shutdown
          @channel.close
        end

        private

        def dispatch_run(run_descriptor)
          pid = fork { run_specs(run_descriptor) }
          result = @channel.receive
          Process.wait(pid)
          raise BisectFailedError.for_failed_spec_run(result) if result.is_a?(String)

          ExampleSetDescriptor.new(*result)
        end

        def run_specs(run_descriptor)
          spec_output = StringIO.new
          $stdout = $stderr = spec_output
          formatter = CaptureFormatter.new(run_descriptor.failed_example_ids)
          results = @spec_runner.run_bisect_subset(run_descriptor.all_example_ids, formatter, spec_output)
          if results.nil? || results.all_example_ids.empty?
            @channel.send(spec_output.string)
          else
            @channel.send([results.all_example_ids, results.failed_example_ids])
          end
        ensure
          exit!(0)
        end
      end

      class ExampleMinimizer
        attr_reader :shell_command, :runner, :all_example_ids, :failed_example_ids
        attr_accessor :remaining_ids

        ExampleRange = Struct.new(:start, :finish) do
          def description
            start == finish ? "example #{start}" : "examples #{start}-#{finish}"
          end
        end

        def initialize(shell_command, runner, notifier)
          @shell_command = shell_command
          @runner = runner
          @notifier = notifier
        end

        def find_minimal_repro
          prep
          _, duration = track_duration { bisect(non_failing_example_ids) }
          notify(:bisect_complete, duration: duration,
                                   original_non_failing_count: non_failing_example_ids.size,
                                   remaining_count: remaining_ids.size)
          remaining_ids + failed_example_ids
        end

        def bisect(candidate_ids)
          notify(:bisect_dependency_check_started)
          if get_expected_failures_for?([])
            notify(:bisect_dependency_check_failed)
            self.remaining_ids = []
            return
          end
          notify(:bisect_dependency_check_passed)
          bisect_over(candidate_ids)
        end

        # Halves the candidates and drops a half the failures do not need,
        # or bisects each half when both are needed.
        def bisect_over(candidate_ids)
          return if candidate_ids.one?

          notify(:bisect_round_started, candidate_range: example_range(candidate_ids),
                                        candidates_count: candidate_ids.size)
          slice_size = (candidate_ids.length / 2.0).ceil
          lhs, rhs = candidate_ids.each_slice(slice_size).to_a
          ids_to_ignore, duration = track_duration do
            [lhs, rhs].find { |ids| get_expected_failures_for?(remaining_ids - ids) }
          end

          if ids_to_ignore
            self.remaining_ids -= ids_to_ignore
            notify(:bisect_round_ignoring_ids, ids_to_ignore: ids_to_ignore,
                                               ignore_range: example_range(ids_to_ignore),
                                               remaining_ids: remaining_ids,
                                               duration: duration)
            bisect_over(candidate_ids - ids_to_ignore)
          else
            notify(:bisect_round_detected_multiple_culprits, duration: duration)
            bisect_over(lhs)
            bisect_over(rhs)
          end
        end

        def currently_needed_ids
          remaining_ids + failed_example_ids
        end

        def repro_command_for_currently_needed_ids
          return shell_command.repro_command_from(currently_needed_ids) if remaining_ids

          "(Not yet enough information to provide any repro command)"
        end

        private

        def example_range(ids)
          ExampleRange.new(non_failing_example_ids.find_index(ids.first) + 1,
                           non_failing_example_ids.find_index(ids.last) + 1)
        end

        def prep
          notify(:bisect_starting, original_cli_args: shell_command.original_cli_args,
                                   bisect_runner: runner.class.name)
          _, duration = track_duration do
            original_results = runner.original_results
            @all_example_ids = original_results.all_example_ids
            @failed_example_ids = original_results.failed_example_ids
            @remaining_ids = non_failing_example_ids
          end

          if @failed_example_ids.empty?
            raise BisectFailedError, "\n\nNo failures found. Bisect only works " \
                                     "in the presence of one or more failing examples."
          end

          notify(:bisect_original_run_complete, failed_example_ids: failed_example_ids,
                                                non_failing_example_ids: non_failing_example_ids,
                                                duration: duration)
        end

        def non_failing_example_ids
          @non_failing_example_ids ||= all_example_ids - failed_example_ids
        end

        def get_expected_failures_for?(ids)
          ids_to_run = all_example_ids & (ids + failed_example_ids)
          notify(:bisect_individual_run_start, command: shell_command.repro_command_from(ids_to_run),
                                               ids_to_run: ids_to_run)
          results, duration = track_duration { runner.run(ids_to_run) }
          notify(:bisect_individual_run_complete, duration: duration, results: results)
          abort_if_ordering_inconsistent(results)
          (failed_example_ids & results.failed_example_ids) == failed_example_ids
        end

        def track_duration
          start = Rex::Core.monotonic_now
          [yield, Rex::Core.monotonic_now - start]
        end

        def abort_if_ordering_inconsistent(results)
          expected_order = all_example_ids & results.all_example_ids
          return if expected_order == results.all_example_ids

          raise BisectFailedError, "\n\nThe example ordering is inconsistent. " \
                                   "`--bisect` relies upon consistent ordering (e.g. by passing " \
                                   "`--seed` if you're using random ordering) to work properly."
        end

        def notify(event, values = {})
          @notifier.publish(event, values)
        end
      end

      class Coordinator
        def self.bisect_with(spec_runner, original_cli_args, formatter)
          new(spec_runner, original_cli_args, formatter).bisect
        end

        def initialize(spec_runner, original_cli_args, formatter)
          @spec_runner = spec_runner
          @shell_command = ShellCommand.new(original_cli_args)
          @notifier = Notifier.new(formatter)
        end

        def bisect
          repro = ForkRunner.start(@shell_command, @spec_runner) do |runner|
            minimizer = ExampleMinimizer.new(@shell_command, runner, @notifier)
            minimizer.find_minimal_repro
            minimizer.repro_command_for_currently_needed_ids
          end
          @notifier.publish(:bisect_repro_command, repro: repro)
          true
        rescue BisectFailedError => error
          @notifier.publish(:bisect_failed, failure_explanation: error.message)
          false
        ensure
          @notifier.publish(:close)
        end
      end
    end

    module Formatters
      # The progress `--bisect` prints.
      class BisectProgressFormatter
        attr_reader :output

        def initialize(output, bisect_runner)
          @output = output
          @bisect_runner = bisect_runner
        end

        def bisect_starting(notification)
          @round_count = 0
          output.puts bisect_started_message(notification)
          output.print "Running suite to find failures..."
        end

        def bisect_original_run_complete(notification)
          failures = Formatters.pluralize(notification.failed_example_ids.size, "failing example")
          non_failures = Formatters.pluralize(notification.non_failing_example_ids.size, "non-failing example")
          output.puts " (#{Formatters.format_duration(notification.duration)})"
          output.puts "Starting bisect with #{failures} and #{non_failures}."
        end

        def bisect_dependency_check_started(_notification)
          output.print "Checking that failure(s) are order-dependent.."
        end

        def bisect_dependency_check_passed(_notification)
          output.puts " failure appears to be order-dependent"
        end

        def bisect_dependency_check_failed(_notification)
          output.puts " failure(s) do not require any non-failures to run first"
          return unless @bisect_runner == :fork

          output.puts
          output.puts "=" * 80
          output.puts "NOTE: this bisect run used `config.bisect_runner = :fork`, which generally"
          output.puts "provides significantly faster bisection runs than the old shell-based runner,"
          output.puts "but may inaccurately report that no non-failures are required. If this result"
          output.puts "is unexpected, consider setting `config.bisect_runner = :shell` and trying again."
          output.puts "=" * 80
        end

        def bisect_round_started(notification, include_trailing_space = true)
          @round_count += 1
          output.print "\nRound #{@round_count}: bisecting over non-failing #{notification.candidate_range.description}"
          output.print " " if include_trailing_space
        end

        def bisect_round_ignoring_ids(notification)
          output.print " ignoring #{notification.ignore_range.description}"
          output.print " (#{Formatters.format_duration(notification.duration)})"
        end

        def bisect_round_detected_multiple_culprits(notification)
          output.print " multiple culprits detected - splitting candidates"
          output.print " (#{Formatters.format_duration(notification.duration)})"
        end

        def bisect_individual_run_complete(_notification)
          output.print "."
        end

        def bisect_complete(notification)
          output.puts "\nBisect complete! Reduced necessary non-failing examples " \
                      "from #{notification.original_non_failing_count} to " \
                      "#{notification.remaining_count} in " \
                      "#{Formatters.format_duration(notification.duration)}."
        end

        def bisect_repro_command(notification)
          output.puts "\nThe minimal reproduction command is:\n  #{notification.repro}"
        end

        def bisect_failed(notification)
          output.puts "\nBisect failed! #{notification.failure_explanation}"
        end

        def close(_notification)
          output.puts
          output.flush if output.respond_to?(:flush)
        end

        private

        def bisect_started_message(notification)
          "Bisect started using options: #{notification.original_cli_args.join(' ').inspect}"
        end
      end

      # `--bisect=verbose`: each command run and each set of ids as well.
      class BisectDebugFormatter < BisectProgressFormatter
        def bisect_original_run_complete(notification)
          output.puts " (#{Formatters.format_duration(notification.duration)})"
          output.puts " - #{describe_ids('Failing examples', notification.failed_example_ids)}"
          output.puts " - #{describe_ids('Non-failing examples', notification.non_failing_example_ids)}"
        end

        def bisect_individual_run_start(notification)
          output.print "\n - Running: #{notification.command}"
        end

        def bisect_individual_run_complete(notification)
          output.print " (#{Formatters.format_duration(notification.duration)})"
        end

        def bisect_dependency_check_passed(_notification)
          output.print "\n - Failure appears to be order-dependent"
        end

        def bisect_dependency_check_failed(_notification)
          output.print "\n - Failure is not order-dependent"
        end

        def bisect_round_started(notification)
          super(notification, false)
        end

        def bisect_round_ignoring_ids(notification)
          output.print "\n - #{describe_ids('Examples we can safely ignore', notification.ids_to_ignore)}"
          output.print "\n - #{describe_ids('Remaining non-failing examples', notification.remaining_ids)}"
        end

        def bisect_round_detected_multiple_culprits(_notification)
          output.print "\n - Multiple culprits detected - splitting candidates"
        end

        private

        def describe_ids(description, ids)
          organized_ids = Bisect::ShellCommand.new([]).__send__(:organize_ids, ids)
          formatted_ids = organized_ids.map { |id| "    - #{id}" }.join("\n")
          "#{description} (#{ids.size}):\n#{formatted_ids}"
        end

        def bisect_started_message(notification)
          "#{super} and bisect runner: #{notification.bisect_runner.inspect}"
        end
      end
    end
  end
end
