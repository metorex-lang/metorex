module Rex
  module Core
    # Drops the frames that belong to Rex and to Ruby's own library, so a
    # failure points at the user's code. Rex's frames are told apart by the
    # path Metorex gives a library it loads from the binary, `<metorex>/...`,
    # which no file on disk can have, and by the directory Rex was loaded
    # from when it was loaded from disk.
    class BacktraceFormatter
      attr_accessor :exclusion_patterns, :inclusion_patterns
      attr_writer :full_backtrace

      RUBY_LIBRARY_PATTERNS = %w[/lib\d*/ruby/ bin/ exe/rspec /lib/bundler/ /exe/bundle:].map { |text| Regexp.new(text) }.freeze

      # Rex's own frames and the frame `metorex spec` runs the suite from,
      # with the frames RSpec leaves out of every backtrace.
      def self.framework_pattern
        patterns = [%r{\A<metorex>/}, /\Ametorex spec:/, "rubygems/core_ext/kernel_require.rb", "<internal:",
                    %r{/lib/ruby/[^/]+/bundled_gems\.rb}]
        patterns << %r{\A#{Regexp.escape(FRAMEWORK_ROOT)}(?:/|\.rb)} if FRAMEWORK_ROOT.start_with?("/")
        Regexp.union(*patterns)
      end

      def initialize
        @full_backtrace = false
        @exclusion_patterns = [Regexp.union(self.class.framework_pattern, *RUBY_LIBRARY_PATTERNS)]
        @inclusion_patterns = []
        return unless matches?(@exclusion_patterns, File.join(Dir.getwd, "lib", "foo.rb:13"))

        @inclusion_patterns << Regexp.new(Dir.getwd)
      end

      def full_backtrace?
        @full_backtrace || exclusion_patterns.empty?
      end

      def filter_gem(gem_name)
        exclusion_patterns << %r{/#{gem_name}(-[^/]+)?/}
      end

      def format_backtrace(backtrace, options = {})
        return [] unless backtrace
        return backtrace if options[:full_backtrace] || backtrace.empty?

        backtrace.map { |line| backtrace_line(line) }.compact.tap do |filtered|
          if filtered.empty?
            filtered.concat backtrace
            filtered << ""
            filtered << "  Showing full backtrace because every line was filtered out."
            filtered << "  See docs for RSpec::Configuration#backtrace_exclusion_patterns and"
            filtered << "  RSpec::Configuration#backtrace_inclusion_patterns for more information."
          end
        end
      end

      def backtrace_line(line)
        Rex::Core.relative_path_in_line(line.to_s) unless exclude?(line)
      end

      def exclude?(line)
        return false if @full_backtrace

        matches?(exclusion_patterns, line.to_s) && !matches?(inclusion_patterns, line.to_s)
      end

      private

      def matches?(patterns, line)
        patterns.any? { |pattern| line =~ pattern }
      end
    end

    module Formatters
      def self.format_duration(duration)
        precision = duration < 1 ? 5 : 2
        if duration > 60
          minutes = (duration.round / 60).to_i
          seconds = (duration - minutes * 60)
          "#{pluralize(minutes, 'minute')} #{pluralize(format_seconds(seconds, precision), 'second')}"
        else
          pluralize(format_seconds(duration, precision), "second")
        end
      end

      def self.format_seconds(float, precision = nil)
        precision ||= float < 1 ? 5 : 2
        formatted = format("%.#{precision}f", float)
        strip_trailing_zeroes(formatted)
      end

      def self.strip_trailing_zeroes(string)
        string.sub(/(?:(\..*[^0])0+|\.0+)$/, '\1')
      end

      def self.pluralize(count, string)
        "#{count} #{string}#{'s' unless count.to_f == 1}"
      end

      module ConsoleCodes
        CODES = {
          black: 30, red: 31, green: 32, yellow: 33, blue: 34, magenta: 35, cyan: 36, white: 37,
          bold_black: "1;30", bold_red: "1;31", bold_green: "1;32", bold_yellow: "1;33",
          bold_blue: "1;34", bold_magenta: "1;35", bold_cyan: "1;36", bold_white: "1;37",
          bold: 1
        }.freeze

        CONFIG_COLORS = %i[success_color pending_color failure_color default_color detail_color fixed_color].freeze

        def self.console_code_for(code_or_symbol)
          if CONFIG_COLORS.include?(code_or_symbol)
            console_code_for(Rex.configuration.__send__(code_or_symbol))
          else
            CODES.fetch(code_or_symbol) { console_code_for(:white) }
          end
        end

        def self.wrap(text, code_or_symbol)
          return text unless Rex.configuration.color_enabled?

          "\e[#{console_code_for(code_or_symbol)}m#{text}\e[0m"
        end
      end

      # Builds the numbered block a failure or a pending example prints.
      class ExceptionPresenter
        attr_reader :exception, :example, :description, :message_color, :indentation

        def initialize(exception, example, options = {})
          @exception = exception
          @example = example
          @message_color = options.fetch(:message_color) { :failure_color }
          @description = options.fetch(:description) { example ? example.full_description : "" }
          @detail_formatter = options.fetch(:detail_formatter) { nil }
          @extra_detail_formatter = options.fetch(:extra_detail_formatter) { nil }
          @backtrace_formatter = options.fetch(:backtrace_formatter) { Rex.configuration.backtrace_formatter }
          @indentation = options.fetch(:indentation) { 2 }
          @skip_shared_group_trace = options.fetch(:skip_shared_group_trace) { false }
          @failure_lines = options[:failure_lines]
        end

        def fully_formatted(failure_number)
          lines = fully_formatted_lines(failure_number)
          lines.join("\n") + "\n"
        end

        def fully_formatted_lines(failure_number)
          return fixed_pending_lines(failure_number) if exception.is_a?(Rex::Core::PendingExampleFixedError)

          lines = [
            encoded_description(formatted_description(failure_number)),
            detail_formatter_line(failure_number),
            *formatted_message_and_backtrace(failure_number)
          ].compact
          header, *body = lines
          indent = " " * (indentation + (failure_number ? failure_number.to_s.length + 2 : 0))
          ["", header] + body.map { |line| line.empty? ? line : indent + line }
        end

        # A pending example that passed: named FIXED, with the message and the
        # example's location rather than a backtrace.
        def fixed_pending_lines(failure_number)
          indent = " " * (indentation + (failure_number ? failure_number.to_s.length + 2 : 0))
          [
            "",
            "#{formatted_description(failure_number)} FIXED",
            indent + ConsoleCodes.wrap(exception.message, :fixed_color),
            indent + ConsoleCodes.wrap("# #{example.location}", :detail_color)
          ]
        end

        def failure_lines
          @failure_lines ||= begin
            lines = []
            lines.concat(failure_slash_error_lines)
            sections = [failure_slash_error_lines, exception_lines]
            lines << "" if sections.any? { |section| section.size > 1 } && !exception_lines.first.to_s.empty?
            lines.concat(exception_lines)
            lines
          end
        end

        def formatted_backtrace(error = exception)
          @backtrace_formatter.format_backtrace(error.backtrace || []) + formatted_cause(error)
        end

        # The exception at the end of `error`'s chain of causes: its class,
        # its message, and the first line of its backtrace, or every line
        # with `config.full_cause_backtrace`.
        def formatted_cause(error)
          return [] unless error.cause

          last_cause = final_exception(error, [error])
          lines = ["------------------", "--- Caused by: ---"]
          lines << "#{exception_class_name(last_cause)}:" unless exception_class_name(last_cause) =~ /RSpec|\ARex::/
          exception_message_string(last_cause).split("\n").each { |line| lines << "  #{line}" }
          unless last_cause.backtrace.nil? || last_cause.backtrace.empty?
            frames = @backtrace_formatter.format_backtrace(last_cause.backtrace)
            frames = [frames[0]] unless Rex.configuration.full_cause_backtrace
            frames.each { |frame| lines << "  #{frame}" }
          end
          lines
        end

        def final_exception(error, previous = [])
          cause = error.cause
          if cause && Exception === cause && !previous.include?(cause)
            previous << cause
            final_exception(cause, previous)
          else
            error
          end
        end

        def colorized_formatted_backtrace(error = exception)
          formatted_backtrace(error).map { |frame| ConsoleCodes.wrap("# #{frame}", :detail_color) }
        end

        def message_lines
          lines = failure_lines.map { |line| ConsoleCodes.wrap(line, message_color) }
          add_shared_group_lines(lines)
        end

        # Where each shared group the example came from was included.
        def add_shared_group_lines(lines)
          return lines if @skip_shared_group_trace || example.nil?

          (example.metadata[:shared_group_inclusion_backtrace] || []).each do |frame|
            lines << ConsoleCodes.wrap(frame.description, :default_color)
          end
          lines
        end

        private

        def detail_formatter_line(failure_number)
          @detail_formatter&.call(example, @exception, failure_number)
        end

        def formatted_description(failure_number)
          "#{" " * indentation}#{failure_number}) #{description}"
        end

        def encoded_description(text)
          text
        end

        def formatted_message_and_backtrace(failure_number = nil)
          if multiple_exceptions?
            multiple_exception_lines(failure_number)
          else
            message_lines + colorized_formatted_backtrace
          end
        end

        def multiple_exceptions?
          exception.respond_to?(:all_exceptions) && exception.respond_to?(:aggregation_block_label)
        end

        # A failure aggregated from several: the summary and the backtrace of
        # the block that collected them, then each failure numbered under the
        # example's own number, as `54.1)`.
        def multiple_exception_lines(failure_number)
          hide_backtrace = exception.respond_to?(:aggregation_metadata) && exception.aggregation_metadata[:hide_backtrace]
          # With the backtrace hidden the failures follow the summary at once,
          # so it ends in a colon. Otherwise the backtrace comes between them.
          summary = if hide_backtrace
                      "Got #{exception.exception_count_description}:"
                    elsif exception.respond_to?(:summary)
                      "#{exception.summary}."
                    else
                      "Got #{exception.all_exceptions.size} failures."
                    end
          lines = [ConsoleCodes.wrap(summary, message_color)]
          shared = hide_backtrace ? [] : colorized_formatted_backtrace
          lines.concat(shared)
          exception.all_exceptions.each_with_index do |error, index|
            number = "#{failure_number}.#{index + 1}) "
            sub = ExceptionPresenter.new(error, example, indentation: 0, message_color: message_color)
            # The frames the block that collected the failures already lists
            # are left off each failure's own backtrace.
            own = sub.colorized_formatted_backtrace
            own = own[0...-1] while !own.empty? && shared.include?(own.last)
            sub_lines = sub.message_lines + own
            lines << ""
            lines << "#{number}#{sub_lines.first}"
            sub_lines.drop(1).each { |line| lines << (line.empty? ? line : (" " * number.length) + line) }
          end
          lines
        end

        def exception_class_name(error = exception)
          name = error.class.name.to_s
          name = "(anonymous error class)" if name.empty?
          name
        end

        # RSpec leaves its own failure classes unnamed in the report, which
        # covers Rex's under either name.
        def expectation_failure?(error = exception)
          error.is_a?(Rex::ExpectationNotMet) || error.class.name.to_s.match?(/RSpec|\ARex::/)
        end

        def exception_lines
          @exception_lines ||= begin
            lines = []
            lines << "#{exception_class_name}:" unless expectation_failure?
            exception_message_string(exception).split("\n").each do |line|
              lines << (line.empty? ? line : "  #{line}")
            end
            lines
          end
        end

        def exception_message_string(error)
          case error
          when SyntaxError then error.detailed_message.to_s
          else error.message.to_s
          end
        rescue Exception => message_error
          "A #{error.class} for which `exception.message.to_s` raises #{message_error.class}."
        end

        def failure_slash_error_lines
          context = Rex.configuration.failure_context_lines
          if context.positive? && (lines = lines_with_context(context))
            return lines
          end

          lines = read_failed_lines
          if lines.size == 1
            lines[0] = "Failure/Error: #{lines[0].strip}"
          else
            least_indentation = SnippetExtractor.least_indentation_from(lines)
            lines = lines.map { |line| line.sub(/^#{least_indentation}/, "  ") }
            lines.unshift("Failure/Error:")
          end
          lines
        end

        # The failing statement's lines numbered, each marked `>`, with
        # `context` lines before and after it.
        def lines_with_context(context)
          frame = find_failed_line
          file_path, line_number = frame&.match(/\A(.+?):(\d+)/)&.captures
          return nil unless file_path && File.exist?(file_path)

          require "rex/source"
          first, last = Rex::Source.__statement_lines__(File.expand_path(file_path), line_number.to_i)
          first ||= line_number.to_i
          last ||= line_number.to_i
          source = Rex.world.source_lines_for(file_path)
          shown = [first - context, 1].max..[last + context, source.size].min
          width = shown.end.to_s.length
          ["Failure/Error:"] + shown.map do |number|
            marker = (first..last).cover?(number) ? ">" : " "
            "#{marker} #{number.to_s.rjust(width)}| #{source[number - 1]}".rstrip
          end
        end

        def read_failed_lines
          frame = find_failed_line
          return ["Unable to find matching line from backtrace"] unless frame

          file_path, line_number = frame.match(/\A(.+?):(\d+)/)&.captures
          return ["Unable to find matching line from backtrace"] unless file_path

          require "rex/source"
          max_line_count = Rex.configuration.max_displayed_failure_line_count
          SnippetExtractor.extract_expression_lines_at(file_path, line_number.to_i, max_line_count)
        rescue SnippetExtractor::NoSuchFileError
          ["Unable to find #{file_path} to read failed line"]
        rescue SnippetExtractor::NoSuchLineError
          ["Unable to find matching line in #{file_path}"]
        end

        # The first frame in a loaded spec file or under one of the project's
        # source directories.
        def find_failed_line
          configuration = Rex.configuration
          line_regex = configuration.in_project_source_dir_regex
          loaded_spec_files = configuration.loaded_spec_files
          reduced_backtrace = (exception.backtrace || []).reject { |line| line.start_with?("<internal:") }
          reduced_backtrace.find do |line|
            next unless (line_path = line[/(.+?):(\d+)(|:\d+)/, 1])

            path = File.expand_path(line_path)
            loaded_spec_files.include?(path) || path =~ line_regex
          end || reduced_backtrace.first
        end
      end

      # The notification objects formatters receive.
      StartNotification = Struct.new(:count, :load_time)
      ExampleNotification = Struct.new(:example) do
        # The notification for the example's outcome: a failure's carries
        # its message and backtrace.
        def self.for(example)
          if example.execution_result.status == :failed
            Reporter.failure_notification_for(example)
          else
            new(example)
          end
        end
      end
      GroupNotification = Struct.new(:group)
      MessageNotification = Struct.new(:message)
      SeedNotification = Struct.new(:seed, :used) do
        def seed_used?
          used
        end

        def fully_formatted
          "\nRandomized with seed #{seed}\n"
        end
      end
      NullNotification = Struct.new(:nothing)

      class FailedExampleNotification
        attr_reader :example

        def initialize(example, presenter)
          @example = example
          @presenter = presenter
        end

        def exception
          @presenter.exception
        end

        def description
          @example.full_description
        end

        def message_lines
          @presenter.message_lines
        end

        def colorized_message_lines(_colorizer = ConsoleCodes)
          @presenter.message_lines
        end

        def formatted_backtrace
          @presenter.formatted_backtrace
        end

        def colorized_formatted_backtrace(_colorizer = ConsoleCodes)
          @presenter.colorized_formatted_backtrace
        end

        def fully_formatted(failure_number, _colorizer = ConsoleCodes)
          @presenter.fully_formatted(failure_number)
        end

        def fully_formatted_lines(failure_number, _colorizer = ConsoleCodes)
          @presenter.fully_formatted_lines(failure_number)
        end
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
          examples.map { |example| ExampleNotification.for(example) }
        end

        def failure_notifications
          @failure_notifications ||= failed_examples.map { |example| Reporter.failure_notification_for(example) }
        end

        def pending_notifications
          @pending_notifications ||= pending_examples.map { |example| Reporter.pending_notification_for(example) }
        end

        def fully_formatted_failed_examples(_colorizer = ConsoleCodes)
          formatted = +"\nFailures:\n"
          failure_notifications.each_with_index do |failure, index|
            formatted << failure.fully_formatted(index + 1)
          end
          formatted
        end

        def fully_formatted_pending_examples(_colorizer = ConsoleCodes)
          return if Rex.configuration.pending_failure_output == :skip

          formatted = +"\nPending: (Failures listed here are expected and do not affect your suite's status)\n"
          pending_notifications.each_with_index do |notification, index|
            formatted << notification.fully_formatted(index + 1)
          end
          formatted
        end
      end

      # A pending example's block: its reason and location, and the failure
      # it raised when it was allowed to run.
      class PendingExampleNotification
        attr_reader :example

        def initialize(example)
          @example = example
        end

        def fully_formatted(pending_number, _colorizer = ConsoleCodes)
          result = example.execution_result
          return skipped_fully_formatted(pending_number) unless result.pending_exception

          lines = ["", "  #{pending_number}) #{example.full_description}"]
          indent = " " * (5 + pending_number.to_s.length - 1)
          lines << indent + ConsoleCodes.wrap("# #{result.pending_message}", :detail_color)
          presenter = ExceptionPresenter.new(result.pending_exception, example, message_color: :pending_color)
          presenter.message_lines.each { |line| lines << (line.empty? ? line : indent + line) }
          unless Rex.configuration.pending_failure_output == :no_backtrace
            presenter.colorized_formatted_backtrace.each { |line| lines << indent + line }
          end
          lines.join("\n") + "\n"
        end

        private

        # A skipped example, which raised nothing: its heading in the
        # pending color, its reason, and where it is.
        def skipped_fully_formatted(pending_number)
          [
            ConsoleCodes.wrap("\n  #{pending_number}) #{example.full_description}", :pending_color),
            "\n     ",
            ConsoleCodes.wrap("# #{example.execution_result.pending_message}", :detail_color),
            "\n",
            ConsoleCodes.wrap("     # #{example.location}\n", :detail_color)
          ].join
        end
      end

      class SummaryNotification
        attr_reader :duration, :examples, :failed_examples, :pending_examples, :load_time, :errors_outside_of_examples_count

        def initialize(duration, examples, failed_examples, pending_examples, load_time, errors_outside_of_examples_count)
          @duration = duration
          @examples = examples
          @failed_examples = failed_examples
          @pending_examples = pending_examples
          @load_time = load_time
          @errors_outside_of_examples_count = errors_outside_of_examples_count
        end

        def example_count
          examples.size
        end

        def failure_count
          failed_examples.size
        end

        def pending_count
          pending_examples.size
        end

        def formatted_duration
          Formatters.format_duration(duration)
        end

        def formatted_load_time
          Formatters.format_duration(load_time)
        end

        def totals_line
          summary = Formatters.pluralize(example_count, "example") + ", " + Formatters.pluralize(failure_count, "failure")
          summary += ", #{pending_count} pending" if pending_count > 0
          if errors_outside_of_examples_count > 0
            summary += ", "
            summary += Formatters.pluralize(errors_outside_of_examples_count, "error")
            summary += " occurred outside of examples"
          end
          summary
        end

        def colorized_totals_line(_colorizer = ConsoleCodes)
          if failure_count > 0 || errors_outside_of_examples_count > 0
            ConsoleCodes.wrap(totals_line, :failure_color)
          elsif pending_count > 0
            ConsoleCodes.wrap(totals_line, :pending_color)
          else
            ConsoleCodes.wrap(totals_line, :success_color)
          end
        end

        def colorized_rerun_commands(_colorizer = ConsoleCodes)
          "Failed examples:\n\n" + failed_examples.map do |example|
            ConsoleCodes.wrap("rspec #{rerun_argument_for(example)}", :failure_color) + " " +
              ConsoleCodes.wrap("# #{example.full_description}", :detail_color)
          end.join("\n")
        end

        def rerun_argument_for(example)
          location = example.location_rerun_argument
          return location unless duplicate_rerun_locations.include?(location)

          conditionally_quote(example.id)
        end

        def duplicate_rerun_locations
          @duplicate_rerun_locations ||= Rex.world.all_examples.map(&:location_rerun_argument)
                                            .tally.select { |_, count| count > 1 }.keys
        end

        # An id holds brackets, which a shell would read as a glob, so it is
        # quoted the way RSpec quotes it.
        def conditionally_quote(id)
          return id unless id.match?(/[\[\]'" ]/)

          "'#{id.gsub("'", "\\\\'")}'"
        end

        def fully_formatted(_colorizer = ConsoleCodes)
          formatted = "\nFinished in #{formatted_duration} (files took #{formatted_load_time} to load)\n" \
                      "#{colorized_totals_line}\n"
          formatted += "\n" + colorized_rerun_commands unless failed_examples.empty?
          formatted
        end
      end

      class ProfileNotification
        attr_reader :duration, :examples, :number_of_examples

        def initialize(duration, examples, number_of_examples, example_groups)
          @duration = duration
          @examples = examples
          @number_of_examples = number_of_examples
          @example_groups = example_groups
        end

        def slowest_examples
          @slowest_examples ||= examples.sort_by { |example| -example.execution_result.run_time }.first(number_of_examples)
        end

        def slow_duration
          @slow_duration ||= slowest_examples.inject(0.0) { |sum, example| sum + example.execution_result.run_time }
        end

        def percentage
          @percentage ||= begin
            time_taken = slow_duration / duration
            format("%.1f", (time_taken.nan? ? 0.0 : time_taken) * 100)
          end
        end

        def slowest_groups
          @slowest_groups ||= calculate_slowest_groups
        end

        private

        # Each top-level group by its average example time, slowest first,
        # once there is more than one group to rank.
        def calculate_slowest_groups
          return {} if @example_groups.keys.length <= 1

          @example_groups.each_value do |hash|
            hash[:average] = hash[:total_time].to_f / hash[:count]
          end
          groups = @example_groups.sort_by { |_, hash| -hash[:average] }.first(number_of_examples)
          groups.map { |group, data| [group.location, data] }
        end
      end

      # The formatter classes and the events each one listens for, as
      # `Formatters.register` records them.
      class Loader
        def self.formatters
          @formatters ||= {}
        end
      end

      NullColorizer = Module.new do
        def self.wrap(line, _color)
          line
        end
      end

      def self.register(formatter_class, *notifications)
        Loader.formatters[formatter_class] = notifications
      end

      def self.notifications_for(formatter_class)
        formatter_class.ancestors.flat_map { |ancestor| Loader.formatters.fetch(ancestor, []) }.uniq
      end

      class BaseFormatter
        Formatters.register self, :start, :example_group_started, :close

        attr_reader :output, :example_group

        def initialize(output)
          @output = output || StringIO.new
          @example_group = nil
        end

        def start(notification)
          @example_count = notification.count
        end

        def example_group_started(notification)
          @example_group = notification.group
        end

        def close(_notification)
          return if output.respond_to?(:closed?) && output.closed?

          output.puts
          output.flush if output.respond_to?(:flush)
        end
      end

      class BaseTextFormatter < BaseFormatter
        Formatters.register self, :message, :dump_summary, :dump_failures, :dump_pending, :seed
        def message(notification)
          output.puts notification.message
        end

        def dump_failures(notification)
          return if notification.failure_notifications.empty?

          output.puts notification.fully_formatted_failed_examples
        end

        def dump_summary(summary)
          output.puts summary.fully_formatted
        end

        def seed(notification)
          return unless notification.seed_used?

          output.puts notification.fully_formatted
        end

        def dump_pending(notification)
          return if notification.pending_examples.empty?

          output.puts notification.fully_formatted_pending_examples
        end
      end

      class ProgressFormatter < BaseTextFormatter
        Formatters.register self, :example_passed, :example_pending, :example_failed, :start_dump
        def example_passed(_notification)
          output.print ConsoleCodes.wrap(".", :success_color)
        end

        def example_pending(_notification)
          output.print ConsoleCodes.wrap("*", :pending_color)
        end

        def example_failed(_notification)
          output.print ConsoleCodes.wrap("F", :failure_color)
        end

        def start_dump(_notification)
          output.puts
        end
      end

      class DocumentationFormatter < BaseTextFormatter
        Formatters.register self, :example_started, :example_group_started, :example_group_finished,
                            :example_passed, :example_pending, :example_failed
        def initialize(output)
          super
          @group_level = 0
          @example_running = false
          @messages = []
        end

        def example_started(_notification)
          @example_running = true
        end

        def example_group_started(notification)
          output.puts if @group_level.zero?
          output.puts "#{current_indentation}#{notification.group.description.strip}"
          @group_level += 1
        end

        def example_group_finished(_notification)
          @group_level -= 1 if @group_level > 0
        end

        def example_passed(passed)
          output.puts passed_output(passed.example)
          flush_messages
          @example_running = false
        end

        def example_pending(pending)
          output.puts pending_output(pending.example, pending.example.execution_result.pending_message)
          flush_messages
          @example_running = false
        end

        def example_failed(failure)
          output.puts failure_output(failure.example)
          flush_messages
          @example_running = false
        end

        def message(notification)
          if @example_running
            @messages << notification.message
          else
            output.puts "#{current_indentation}#{notification.message}"
          end
        end

        private

        def flush_messages
          @messages.each { |message| output.puts "#{current_indentation(1)}#{message}" }
          @messages.clear
        end

        def passed_output(example)
          ConsoleCodes.wrap("#{current_indentation}#{example.description.strip}", :success_color)
        end

        def pending_output(example, message)
          ConsoleCodes.wrap("#{current_indentation}#{example.description.strip} (PENDING: #{message})", :pending_color)
        end

        def failure_output(example)
          ConsoleCodes.wrap("#{current_indentation}#{example.description.strip} (FAILED - #{next_failure_index})", :failure_color)
        end

        def next_failure_index
          @next_failure_index ||= 0
          @next_failure_index += 1
        end

        def current_indentation(offset = 0)
          "  " * (@group_level + offset)
        end
      end

      # Prints messages when no other formatter listens for them.
      class FallbackMessageFormatter
        Formatters.register self, :message

        attr_reader :output

        def initialize(output)
          @output = output
        end

        def message(notification)
          output.puts notification.message
        end
      end

      # `--format failures`: one `location:description` line per failure.
      class FailureListFormatter < BaseFormatter
        Formatters.register self, :example_failed, :dump_profile, :message

        def example_failed(failure)
          output.puts "#{failure.example.location}:#{failure.example.description}"
        end

        def dump_profile(_profile); end

        def message(_message); end
      end

      # `--profile`: the slowest examples and groups.
      class ProfileFormatter
        Formatters.register self, :dump_profile

        attr_reader :output

        def initialize(output)
          @output = output
        end

        def dump_profile(profile)
          dump_profile_slowest_examples(profile)
          dump_profile_slowest_example_groups(profile)
        end

        private

        def dump_profile_slowest_examples(profile)
          @output.puts "\nTop #{profile.slowest_examples.size} slowest " \
                       "examples (#{Formatters.format_seconds(profile.slow_duration)} " \
                       "seconds, #{profile.percentage}% of total time):\n"
          profile.slowest_examples.each do |example|
            @output.puts "  #{example.full_description}"
            @output.puts "    #{bold(Formatters.format_seconds(example.execution_result.run_time))} " \
                         "#{bold('seconds')} #{example.location}"
          end
        end

        def dump_profile_slowest_example_groups(profile)
          return if profile.slowest_groups.empty?

          @output.puts "\nTop #{profile.slowest_groups.size} slowest example groups:"
          profile.slowest_groups.each do |location, hash|
            average = "#{bold(Formatters.format_seconds(hash[:average]))} #{bold('seconds')} average"
            total = "#{Formatters.format_seconds(hash[:total_time])} seconds"
            count = Formatters.pluralize(hash[:count], "example")
            @output.puts "  #{hash[:description]}"
            @output.puts "    #{average} (#{total} / #{count}) #{location}"
          end
        end

        def bold(text)
          ConsoleCodes.wrap(text, :bold)
        end
      end

      BUILT_IN = {
        "progress" => ProgressFormatter, "p" => ProgressFormatter,
        "documentation" => DocumentationFormatter, "d" => DocumentationFormatter, "doc" => DocumentationFormatter,
        "failures" => FailureListFormatter, "f" => FailureListFormatter
      }

      # The formatter `formatter` names, writing to `output`: the run's
      # output stream when none is given, or the file a String names. An
      # instance is used as it is.
      def self.build(formatter, output, configuration)
        return formatter unless formatter.is_a?(Class) || formatter.is_a?(String) || formatter.is_a?(Symbol)

        stream = case output
                 when nil then configuration.output_stream
                 when String
                   directory = File.dirname(output)
                   FileUtils.mkdir_p(directory) unless File.directory?(directory)
                   File.new(output, "w")
                 else output
                 end
        formatter_class_for(formatter).new(stream)
      end

      # Formatters loaded only when a run names them.
      LOADED_ON_DEMAND = {
        "json" => ["rex/core/json_formatter", :JsonFormatter], "j" => ["rex/core/json_formatter", :JsonFormatter],
        "html" => ["rex/core/html_formatter", :HtmlFormatter], "h" => ["rex/core/html_formatter", :HtmlFormatter]
      }.freeze

      def self.formatter_class_for(name)
        return name if name.is_a?(Class)

        if (path, constant = LOADED_ON_DEMAND[name.to_s])
          require path
          return const_get(constant)
        end

        BUILT_IN.fetch(name.to_s) do
          if name.is_a?(String) && name =~ /\A[A-Z][a-zA-Z0-9_:]*\z/
            custom_formatter(name)
          else
            raise ArgumentError, "Formatter '#{name}' unknown - maybe you meant 'documentation' or 'progress'?."
          end
        end
      end

      # A formatter named by its constant. A constant not yet defined is
      # looked for in the file its name gives, `MyFormatter` in
      # `my_formatter`, before the lookup is tried again.
      def self.custom_formatter(name)
        name.sub(/\A::/, "").split("::").inject(Object) { |scope, constant| scope.const_get(constant) }
      rescue NameError
        require(path_for(name)) ? retry : raise
      end

      def self.path_for(constant_name)
        word = constant_name.to_s.gsub("::", "/")
        word = word.gsub(/([A-Z]+)([A-Z][a-z])/, '\\1_\\2')
        word = word.gsub(/([a-z\d])([A-Z])/, '\\1_\\2')
        word.tr("-", "_").downcase.sub(%r{(^|/)r_spec($|/)}, '\\1rspec\\2')
      end
    end

    # Hands every event to the formatters, and keeps the totals.
    class Reporter
      attr_reader :examples, :failed_examples, :pending_examples, :formatters

      def self.failure_notification_for(example)
        presenter = Formatters::ExceptionPresenter.new(example.execution_result.exception, example)
        Formatters::FailedExampleNotification.new(example, presenter)
      end

      def self.pending_notification_for(example)
        Formatters::PendingExampleNotification.new(example)
      end

      # Each top-level group's example count and run time, for `--profile`.
      class Profiler
        NOTIFICATIONS = %i[example_group_started example_group_finished example_started].freeze

        attr_reader :example_groups

        def initialize
          @example_groups = Hash.new { |hash, group| hash[group] = { count: 0 } }
        end

        def example_group_started(notification)
          return unless notification.group.top_level?

          @example_groups[notification.group][:start] = Rex::Core.monotonic_now
          @example_groups[notification.group][:description] = notification.group.top_level_description
        end

        def example_group_finished(notification)
          return unless notification.group.top_level?

          group = @example_groups[notification.group]
          return unless group.key?(:start)

          group[:total_time] = Rex::Core.monotonic_now - group[:start]
        end

        def example_started(notification)
          group = notification.example.example_group.parent_groups.last
          @example_groups[group][:count] += 1
        end
      end

      def initialize(configuration)
        @configuration = configuration
        @formatters = []
        @listeners = Hash.new { |hash, event| hash[event] = [] }
        @profiler = Profiler.new
        register_listener(@profiler, *Profiler::NOTIFICATIONS)
        @examples = []
        @failed_examples = []
        @pending_examples = []
        @non_example_exception_count = 0
        @duration = 0
        @start = nil
        @load_time = 0
      end

      def register_listener(listener, *notifications)
        notifications.each { |notification| @listeners[notification.to_sym] << listener }
        true
      end

      def registered_listeners(notification)
        @listeners.key?(notification) ? @listeners[notification] : []
      end

      # A formatter hears the events its class registered with
      # `Formatters.register`. A message sent before any formatter is
      # attached, such as an error requiring a file named with `--require`,
      # waits for the first one.
      def add_formatter(formatter)
        unless Formatters::Loader.formatters.key?(formatter.class)
          @configuration.error_stream.puts "The #{formatter.class} formatter uses the deprecated formatter " \
                                           "interface not supported directly by RSpec 3."
          return
        end
        return if duplicate_formatter_exists?(formatter)

        register_listener(formatter, *Formatters.notifications_for(formatter.class))
        @formatters << formatter
        held = @held_messages || []
        @held_messages = []
        held.each { |text| message(text) }
      end

      # After the configured formatters: one to print messages when none of
      # them does, and the profile when `--profile` is on and none prints it.
      # The deprecation stream, with a path opened as a file.
      def deprecation_output
        stream = @configuration.deprecation_stream
        return stream unless stream.is_a?(String)

        directory = File.dirname(stream)
        FileUtils.mkdir_p(directory) unless File.directory?(directory)
        File.new(stream, "w")
      end

      def set_up?
        @set_up ? true : false
      end

      def setup_default(output_stream)
        @set_up = true
        unless @formatters.any? { |formatter| formatter.is_a?(Formatters::DeprecationFormatter) }
          add_formatter(Formatters::DeprecationFormatter.new(deprecation_output, output_stream))
        end
        @configuration.pending_deprecations.each { |data| deprecation(data) }
        @configuration.pending_deprecations.clear
        add_formatter(Formatters::FallbackMessageFormatter.new(output_stream)) if registered_listeners(:message).empty?
        return unless @configuration.profile_examples?
        return unless registered_listeners(:dump_profile).empty?

        add_formatter(Formatters::ProfileFormatter.new(output_stream))
      end

      def start(expected_example_count, load_time)
        @start = Rex::Core.monotonic_now
        @load_time = load_time
        notify(:seed, Formatters::SeedNotification.new(@configuration.seed, @configuration.seed_used?))
        notify(:start, Formatters::StartNotification.new(expected_example_count, load_time))
      end

      def report(expected_example_count, load_time)
        start(expected_example_count, load_time)
        begin
          yield self
        ensure
          finish
        end
      end

      def example_group_started(group)
        notify(:example_group_started, Formatters::GroupNotification.new(group)) unless group.descendant_filtered_examples.empty?
      end

      def example_group_finished(group)
        notify(:example_group_finished, Formatters::GroupNotification.new(group)) unless group.descendant_filtered_examples.empty?
      end

      def example_started(example)
        @examples << example
        notify(:example_started, Formatters::ExampleNotification.new(example))
      end

      def example_finished(example)
        notify(:example_finished, Formatters::ExampleNotification.new(example))
      end

      def example_passed(example)
        notify(:example_passed, Formatters::ExampleNotification.new(example))
      end

      def example_failed(example)
        @failed_examples << example
        notify(:example_failed, Reporter.failure_notification_for(example))
      end

      def example_pending(example)
        @pending_examples << example
        notify(:example_pending, Formatters::ExampleNotification.new(example))
      end

      def close
        notify(:close, Formatters::NullNotification.new(nil))
      end

      def duplicate_formatter_exists?(new_formatter)
        @formatters.any? do |formatter|
          formatter.class == new_formatter.class &&
            (!formatter.respond_to?(:output) || !new_formatter.respond_to?(:output) || formatter.output == new_formatter.output)
        end
      end

      def message(message)
        if @formatters.empty?
          (@held_messages ||= []) << message
        else
          notify(:message, Formatters::MessageNotification.new(message))
        end
      end

      def notify_non_example_exception(exception, context_description)
        @non_example_exception_count += 1
        Rex.world.non_example_failure = true
        presenter = Formatters::ExceptionPresenter.new(exception, nil, description: context_description, indentation: 0)
        lines = ["", context_description, *presenter.message_lines, *presenter.colorized_formatted_backtrace]
        message(lines.join("\n"))
      end

      def fail_fast_limit_met?
        limit = @configuration.fail_fast
        return false unless limit

        limit = 1 if limit == true
        @failed_examples.size >= limit
      end

      def finish
        examples_notification = Formatters::ExamplesNotification.new(self)
        @duration = Rex::Core.monotonic_now - @start
        notify(:stop, examples_notification)
        notify(:start_dump, Formatters::NullNotification.new(nil))
        notify(:dump_pending, examples_notification)
        notify(:dump_failures, examples_notification)
        notify(:deprecation_summary, Formatters::NullNotification.new(nil))
        unless mute_profile_output?
          notify(:dump_profile, Formatters::ProfileNotification.new(@duration, @examples, @configuration.profile_examples,
                                                                    @profiler.example_groups))
        end
        notify(:dump_summary, Formatters::SummaryNotification.new(@duration, @examples, @failed_examples, @pending_examples, @load_time, @non_example_exception_count))
        notify(:seed, Formatters::SeedNotification.new(@configuration.seed, @configuration.seed_used?))
      ensure
        notify(:close, Formatters::NullNotification.new(nil))
      end

      def non_example_exception_count
        @non_example_exception_count
      end

      def notify(event, notification)
        registered_listeners(event).each do |formatter|
          formatter.__send__(event, notification)
        end
      end

      private

      # The profile is left out when `--fail-fast` stopped the run.
      def mute_profile_output?
        !@configuration.profile_examples? || fail_fast_limit_met?
      end
    end

    # RSpec's name for the module holding the notification objects.
    Notifications = Formatters
  end
end
