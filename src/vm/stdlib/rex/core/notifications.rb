require "rex/core/formatters/console_codes"
require "rex/core/formatters/exception_presenter"
require "rex/core/formatters/helpers"
require "rex/core/shell_escape"

module Rex::Core
  module Notifications
    module NullColorizer
      module_function

      def wrap(line, _code_or_symbol)
        line
      end
    end

    StartNotification = Struct.new(:count, :load_time)

    ExampleNotification = Struct.new(:example)
    class ExampleNotification
      def self.for(example)
        execution_result = example.execution_result

        return SkippedExampleNotification.new(example) if execution_result.example_skipped?
        return new(example) unless execution_result.status == :pending || execution_result.status == :failed

        klass = if execution_result.pending_fixed?
                  PendingExampleFixedNotification
                elsif execution_result.status == :pending
                  PendingExampleFailedAsExpectedNotification
                else
                  FailedExampleNotification
                end

        klass.new(example)
      end

      private_class_method :new
    end

    class ExamplesNotification
      def initialize(reporter)
        @reporter = reporter
      end

      def examples
        @reporter.examples
      end

      def failed_examples
        @reporter.failed_examples
      end

      def pending_examples
        @reporter.pending_examples
      end

      def notifications
        @notifications ||= format_examples(examples)
      end

      def failure_notifications
        @failed_notifications ||= format_examples(failed_examples)
      end

      def pending_notifications
        @pending_notifications ||= format_examples(pending_examples)
      end

      def fully_formatted_failed_examples(colorizer=::Rex::Core::Formatters::ConsoleCodes)
        formatted = "\nFailures:\n"

        failure_notifications.each_with_index do |failure, index|
          formatted += failure.fully_formatted(index.next, colorizer)
        end

        formatted
      end

      def fully_formatted_pending_examples(colorizer=::Rex::Core::Formatters::ConsoleCodes)
        return if Rex.configuration.pending_failure_output == :skip

        formatted = "\nPending: (Failures listed here are expected and do not affect your suite's status)\n".dup

        pending_notifications.each_with_index do |notification, index|
          formatted << notification.fully_formatted(index.next, colorizer)
        end

        formatted
      end

    private

      def format_examples(examples)
        examples.map do |example|
          ExampleNotification.for(example)
        end
      end
    end

    class FailedExampleNotification < ExampleNotification
      public_class_method :new

      def exception
        @exception_presenter.exception
      end

      def description
        @exception_presenter.description
      end

      def message_lines
        @exception_presenter.message_lines
      end

      def colorized_message_lines(colorizer=::Rex::Core::Formatters::ConsoleCodes)
        @exception_presenter.colorized_message_lines(colorizer)
      end

      def formatted_backtrace
        @exception_presenter.formatted_backtrace
      end

      def colorized_formatted_backtrace(colorizer=::Rex::Core::Formatters::ConsoleCodes)
        @exception_presenter.colorized_formatted_backtrace(colorizer)
      end

      def fully_formatted(failure_number, colorizer=::Rex::Core::Formatters::ConsoleCodes)
        @exception_presenter.fully_formatted(failure_number, colorizer)
      end

      def fully_formatted_lines(failure_number, colorizer=::Rex::Core::Formatters::ConsoleCodes)
        @exception_presenter.fully_formatted_lines(failure_number, colorizer)
      end

    private

      def initialize(example, exception_presenter=Formatters::ExceptionPresenter::Factory.new(example).build)
        @exception_presenter = exception_presenter
        super(example)
      end
    end

    class PendingExampleFixedNotification < FailedExampleNotification; end

    class PendingExampleFailedAsExpectedNotification < FailedExampleNotification; end

    class SkippedExampleNotification < ExampleNotification
      public_class_method :new

      def fully_formatted(pending_number, colorizer=::Rex::Core::Formatters::ConsoleCodes)
        formatted_caller = Rex.configuration.backtrace_formatter.backtrace_line(example.location)

        [
          colorizer.wrap("\n  #{pending_number}) #{example.full_description}", :pending),
          "\n     ",
          Formatters::ExceptionPresenter::PENDING_DETAIL_FORMATTER.call(example, colorizer),
          "\n",
          colorizer.wrap("     # #{formatted_caller}\n", :detail)
        ].join("")
      end
    end

    GroupNotification = Struct.new(:group)

    MessageNotification = Struct.new(:message)

    SeedNotification = Struct.new(:seed, :used)
    class SeedNotification
      def seed_used?
        !!used
      end
      private :used

      def fully_formatted
        "\nRandomized with seed #{seed}\n"
      end
    end

    SummaryNotification = Struct.new(:duration, :examples, :failed_examples,
                                     :pending_examples, :load_time,
                                     :errors_outside_of_examples_count)
    class SummaryNotification
      def example_count
        @example_count ||= examples.size
      end

      def failure_count
        @failure_count ||= failed_examples.size
      end

      def pending_count
        @pending_count ||= pending_examples.size
      end

      def totals_line
        summary = Formatters::Helpers.pluralize(example_count, "example") +
          ", " + Formatters::Helpers.pluralize(failure_count, "failure")
        summary += ", #{pending_count} pending" if pending_count > 0
        if errors_outside_of_examples_count > 0
          summary += (
            ", " +
            Formatters::Helpers.pluralize(errors_outside_of_examples_count, "error") +
            " occurred outside of examples"
          )
        end
        summary
      end

      def colorized_totals_line(colorizer=::Rex::Core::Formatters::ConsoleCodes)
        if failure_count > 0 || errors_outside_of_examples_count > 0
          colorizer.wrap(totals_line, Rex.configuration.failure_color)
        elsif pending_count > 0
          colorizer.wrap(totals_line, Rex.configuration.pending_color)
        else
          colorizer.wrap(totals_line, Rex.configuration.success_color)
        end
      end

      def colorized_rerun_commands(colorizer=::Rex::Core::Formatters::ConsoleCodes)
        "\nFailed examples:\n\n" +
        failed_examples.map do |example|
          colorizer.wrap("rspec #{rerun_argument_for(example)}", Rex.configuration.failure_color) + " " +
          colorizer.wrap("# #{example.full_description}",   Rex.configuration.detail_color)
        end.join("\n")
      end

      def formatted_duration
        Formatters::Helpers.format_duration(duration)
      end

      def formatted_load_time
        Formatters::Helpers.format_duration(load_time)
      end

      def fully_formatted(colorizer=::Rex::Core::Formatters::ConsoleCodes)
        formatted = "\nFinished in #{formatted_duration} " \
                    "(files took #{formatted_load_time} to load)\n" \
                    "#{colorized_totals_line(colorizer)}\n"

        unless failed_examples.empty?
          formatted += (colorized_rerun_commands(colorizer) + "\n")
        end

        formatted
      end

    private

      include Rex::Core::ShellEscape

      def rerun_argument_for(example)
        location = example.location_rerun_argument
        return location unless duplicate_rerun_locations.include?(location)
        conditionally_quote(example.id)
      end

      def duplicate_rerun_locations
        @duplicate_rerun_locations ||= begin
          locations = Rex.world.all_examples.map(&:location_rerun_argument)

          Set.new.tap do |s|
            locations.group_by { |l| l }.each do |l, ls|
              s << l if ls.count > 1
            end
          end
        end
      end
    end

    class ProfileNotification
      def initialize(duration, examples, number_of_examples, example_groups)
        @duration = duration
        @examples = examples
        @number_of_examples = number_of_examples
        @example_groups = example_groups
      end
      attr_reader :duration, :examples, :number_of_examples

      def slowest_examples
        @slowest_examples ||=
          examples.sort_by do |example|
            -example.execution_result.run_time
          end.first(number_of_examples)
      end

      def slow_duration
        @slow_duration ||=
          slowest_examples.inject(0.0) do |i, e|
            i + e.execution_result.run_time
          end
      end

      def percentage
        @percentage ||=
          begin
            time_taken = slow_duration / duration
            '%.1f' % ((time_taken.nan? ? 0.0 : time_taken) * 100)
          end
      end

      def slowest_groups
        @slowest_groups ||= calculate_slowest_groups
      end

    private

      def calculate_slowest_groups
        return {} if @example_groups.keys.length <= 1

        @example_groups.each_value do |hash|
          hash[:average] = hash[:total_time].to_f / hash[:count]
        end

        groups = @example_groups.sort_by { |_, hash| -hash[:average] }.first(number_of_examples)
        groups.map { |group, data| [group.location, data] }
      end
    end

    DeprecationNotification = Struct.new(:deprecated, :message, :replacement, :call_site)
    class DeprecationNotification
      private_class_method :new

      def self.from_hash(data)
        new data[:deprecated], data[:message], data[:replacement], data[:call_site]
      end
    end

    class NullNotification
    end

    class CustomNotification < Struct
      def self.for(options={})
        return NullNotification if options.keys.empty?
        new(*options.keys).new(*options.values)
      end
    end
  end
end
