require "rex/core/formatters/console_codes"
require "rex/core/formatters/snippet_extractor"
require "rex/core/formatters/syntax_highlighter"
require "rex/support/encoded_string"

module Rex
  module Core
    module Formatters
      class ExceptionPresenter
        attr_reader :exception, :example, :description, :message_color,
                    :detail_formatter, :extra_detail_formatter, :backtrace_formatter
        private :message_color, :detail_formatter, :extra_detail_formatter, :backtrace_formatter

        def initialize(exception, example, options={})
          @exception               = exception
          @example                 = example
          @message_color           = options.fetch(:message_color)          { Rex.configuration.failure_color }
          @description             = options.fetch(:description)            { example.full_description }
          @detail_formatter        = options.fetch(:detail_formatter)       { Proc.new {} }
          @extra_detail_formatter  = options.fetch(:extra_detail_formatter) { Proc.new {} }
          @backtrace_formatter     = options.fetch(:backtrace_formatter)    { Rex.configuration.backtrace_formatter }
          @indentation             = options.fetch(:indentation, 2)
          @skip_shared_group_trace = options.fetch(:skip_shared_group_trace, false)
          @failure_lines           = options[:failure_lines]
        end

        def message_lines
          add_shared_group_lines(failure_lines, Notifications::NullColorizer)
        end

        def colorized_message_lines(colorizer=::Rex::Core::Formatters::ConsoleCodes)
          add_shared_group_lines(failure_lines, colorizer).map do |line|
            colorizer.wrap line, message_color
          end
        end

        def formatted_backtrace(exception=@exception)
          backtrace_formatter.format_backtrace(exception.backtrace, example.metadata) +
            formatted_cause(exception)
        end

        if Rex::Support::RubyFeatures.supports_exception_cause?
          def formatted_cause(exception)
            last_cause = final_exception(exception, [exception])
            cause = []

            if exception.cause
              cause << '------------------'
              cause << '--- Caused by: ---'
              cause << "#{exception_class_name(last_cause)}:" unless exception_class_name(last_cause) =~ ::Rex::Core::Formatters::ExceptionPresenter::RSPEC_CLASS_NAME

              encoded_string(exception_message_string(last_cause)).split("\n").each do |line|
                cause << "  #{line}"
              end

              unless last_cause.backtrace.nil? || last_cause.backtrace.empty?
                lines = backtrace_formatter.format_backtrace(last_cause.backtrace, example.metadata)
                lines = [lines[0]] unless Rex.configuration.full_cause_backtrace

                lines.each do |line|
                  cause << ("  #{line}")
                end
              end
            end

            cause
          end
        else
          def formatted_cause(_)
            []
          end
        end

        def colorized_formatted_backtrace(colorizer=::Rex::Core::Formatters::ConsoleCodes)
          formatted_backtrace.map do |backtrace_info|
            colorizer.wrap "# #{backtrace_info}", Rex.configuration.detail_color
          end
        end

        def fully_formatted(failure_number, colorizer=::Rex::Core::Formatters::ConsoleCodes)
          lines = fully_formatted_lines(failure_number, colorizer)
          lines.join("\n") << "\n"
        end

        def fully_formatted_lines(failure_number, colorizer)
          lines = [
            encoded_description(description),
            detail_formatter.call(example, colorizer),
            formatted_message_and_backtrace(colorizer),
            extra_detail_formatter.call(failure_number, colorizer),
          ].compact.flatten

          lines = indent_lines(lines, failure_number)
          lines.unshift("")
          lines
        end

      private

        def final_exception(exception, previous=[])
          cause = exception.cause

          if cause && Exception === cause && !previous.include?(cause)
            previous << cause
            final_exception(cause, previous)
          else
            exception
          end
        end

        if String.method_defined?(:encoding)
          def encoding_of(string)
            string.encoding
          end

          def encoded_string(string)
            Rex::Support::EncodedString.new(string, Encoding.default_external)
          end
        else
          def encoding_of(_string)
          end

          def encoded_string(string)
            Rex::Support::EncodedString.new(string)
          end
        end

        def indent_lines(lines, failure_number)
          alignment_basis = ' ' * @indentation
          alignment_basis <<  "#{failure_number}) " if failure_number
          indentation = ' ' * alignment_basis.length

          lines.each_with_index.map do |line, index|
            if index == 0
              "#{alignment_basis}#{line}"
            elsif line.empty?
              line
            else
              "#{indentation}#{line}"
            end
          end
        end

        # RSpec leaves its own failure classes unnamed in the report, which
        # covers Rex's under either name.
        RSPEC_CLASS_NAME = /RSpec|\ARex::/

        def exception_class_name(exception=@exception)
          return exception.remote_class_name if exception.respond_to?(:remote_class_name)

          name = exception.class.name.to_s
          name = "(anonymous error class)" if name == ''
          name
        end

        def failure_lines
          @failure_lines ||= [].tap do |lines|
            lines.concat(failure_slash_error_lines)

            sections = [failure_slash_error_lines, exception_lines]
            if sections.any? { |section| section.size > 1 } && !exception_lines.first.empty?
              lines << ''
            end

            lines.concat(exception_lines)
            lines.concat(extra_failure_lines)
          end
        end

        def failure_slash_error_lines
          context = Rex.configuration.failure_context_lines
          if context.positive? && (lines = lines_with_context(context))
            return lines
          end

          lines = read_failed_lines
          if lines.count == 1
            lines[0] = "Failure/Error: #{lines[0].strip}"
          else
            least_indentation = SnippetExtractor.least_indentation_from(lines)
            lines = lines.map { |line| line.sub(/^#{least_indentation}/, '  ') }
            lines.unshift('Failure/Error:')
          end
          lines
        end

        if SyntaxError.instance_methods.include?(:detailed_message)
          def exception_message_string(exception)
            case exception
            when SyntaxError then exception.detailed_message.to_s
            else
              exception.message.to_s
            end
          rescue Exception => other
            "A #{exception.class} for which `exception.message.to_s` raises #{other.class}."
          end
        else
          def exception_message_string(exception)
            exception.message.to_s
          rescue Exception => other
            "A #{exception.class} for which `exception.message.to_s` raises #{other.class}."
          end
        end

        def exception_lines
          @exception_lines ||= begin
            lines = []
            lines << "#{exception_class_name}:" unless exception_class_name =~ ::Rex::Core::Formatters::ExceptionPresenter::RSPEC_CLASS_NAME
            encoded_string(exception_message_string(exception)).split("\n").each do |line|
              lines << (line.empty? ? line : "  #{line}")
            end
            lines
          end
        end

        def extra_failure_lines
          @extra_failure_lines ||= begin
            lines = Array(example.metadata[:extra_failure_lines])
            unless lines.empty?
              lines.unshift('') unless lines.first == ''
              lines.push('') unless lines.last == ''
            end
            lines
          end
        end

        def add_shared_group_lines(lines, colorizer)
          return lines if @skip_shared_group_trace

          example.metadata[:shared_group_inclusion_backtrace].each do |frame|
            lines << colorizer.wrap(frame.description, Rex.configuration.default_color)
          end

          lines
        end

        # The failing statement's lines numbered, each marked `>`, with
        # `context` lines before and after it, which
        # `config.failure_context_lines` asks for.
        def lines_with_context(context)
          frame = find_failed_line
          file_path, line_number = frame&.match(/\A(.+?):(\d+)/)&.captures
          return nil unless file_path && File.exist?(file_path)

          first, last = Rex::Source.__statement_lines__(File.expand_path(file_path), line_number.to_i)
          first ||= line_number.to_i
          last ||= line_number.to_i
          source = File.read(file_path).split("\n")
          shown = [first - context, 1].max..[last + context, source.size].min
          width = shown.end.to_s.length
          ["Failure/Error:"] + shown.map do |number|
            marker = (first..last).cover?(number) ? ">" : " "
            "#{marker} #{number.to_s.rjust(width)}| #{source[number - 1]}".rstrip
          end
        end

        def read_failed_lines
          matching_line = find_failed_line
          unless matching_line
            return ["Unable to find matching line from backtrace"]
          end

          file_and_line_number = matching_line.match(/(.+?):(\d+)(|:\d+)/)

          unless file_and_line_number
            return ["Unable to infer file and line number from backtrace"]
          end

          file_path, line_number = file_and_line_number[1..2]
          max_line_count = Rex.configuration.max_displayed_failure_line_count
          lines = SnippetExtractor.extract_expression_lines_at(file_path, line_number.to_i, max_line_count)
          Rex.world.syntax_highlighter.highlight(lines)
        rescue SnippetExtractor::NoSuchFileError
          ["Unable to find #{file_path} to read failed line"]
        rescue SnippetExtractor::NoSuchLineError
          ["Unable to find matching line in #{file_path}"]
        rescue SecurityError
          ["Unable to read failed line"]
        end

        def find_failed_line
          line_regex = Rex.configuration.in_project_source_dir_regex
          loaded_spec_files = Rex.configuration.loaded_spec_files

          reduced_backtrace =
            exception_backtrace.reject do |line|
              line.start_with?("<internal:")
            end

          reduced_backtrace.find do |line|
            next unless (line_path = line[/(.+?):(\d+)(|:\d+)/, 1])
            path = File.expand_path(line_path)
            loaded_spec_files.include?(path) || path =~ line_regex
          end || reduced_backtrace.first
        end

        def formatted_message_and_backtrace(colorizer)
          lines = colorized_message_lines(colorizer) + colorized_formatted_backtrace(colorizer)
          encoding = encoding_of("")
          lines.map do |line|
            Rex::Support::EncodedString.new(line, encoding)
          end
        end

        if  String.method_defined?(:encoding)
          def encoded_description(description)
            return if description.nil?
            encoded_string(description)
          end
        else
          def encoded_description(description)
            description
          end
        end

        def exception_backtrace
          exception.backtrace || []
        end

        class Factory
          def build
            ExceptionPresenter.new(@exception, @example, options)
          end

        private

          def initialize(example)
            @example          = example
            @execution_result = example.execution_result
            @exception        = if @execution_result.status == :pending
                                  @execution_result.pending_exception
                                else
                                  @execution_result.exception
                                end
          end

          def options
            with_multiple_error_options_as_needed(@exception, pending_options || {})
          end

          def pending_options
            if @execution_result.pending_fixed?
              {
                :description   => "#{@example.full_description} FIXED",
                :message_color => Rex.configuration.fixed_color,
                :failure_lines => [
                  "Expected pending '#{@execution_result.pending_message}' to fail. No error was raised."
                ]
              }
            elsif @execution_result.status == :pending
              options = {
                :message_color    => Rex.configuration.pending_color,
                :detail_formatter => PENDING_DETAIL_FORMATTER
              }
              if Rex.configuration.pending_failure_output == :no_backtrace
                options[:backtrace_formatter] = EmptyBacktraceFormatter
              end
              options
            end
          end

          def with_multiple_error_options_as_needed(exception, options)
            return options unless multiple_exceptions_error?(exception)

            options = options.merge(
              :failure_lines          => [],
              :extra_detail_formatter => sub_failure_list_formatter(exception, options[:message_color]),
              :detail_formatter       => multiple_exception_summarizer(exception,
                                                                       options[:detail_formatter],
                                                                       options[:message_color])
            )

            return options unless exception.aggregation_metadata[:hide_backtrace]
            options[:backtrace_formatter] = EmptyBacktraceFormatter
            options
          end

          def multiple_exceptions_error?(exception)
            MultipleExceptionError::InterfaceTag === exception
          end

          def multiple_exception_summarizer(exception, prior_detail_formatter, color)
            lambda do |example, colorizer|
              summary = if exception.aggregation_metadata[:hide_backtrace]
                          "Got #{exception.exception_count_description}:"
                        else
                          "#{exception.summary}."
                        end

              summary = colorizer.wrap(summary, color || Rex.configuration.failure_color)
              return summary unless prior_detail_formatter
              [
                prior_detail_formatter.call(example, colorizer),
                summary
              ]
            end
          end

          def sub_failure_list_formatter(exception, message_color)
            common_backtrace_truncater = CommonBacktraceTruncater.new(exception)

            lambda do |failure_number, colorizer|
              FlatMap.flat_map(exception.all_exceptions.each_with_index) do |failure, index|
                options = with_multiple_error_options_as_needed(
                  failure,
                  :description             => nil,
                  :indentation             => 0,
                  :message_color           => message_color || Rex.configuration.failure_color,
                  :skip_shared_group_trace => true
                )

                failure   = common_backtrace_truncater.with_truncated_backtrace(failure)
                presenter = ExceptionPresenter.new(failure, @example, options)
                presenter.fully_formatted_lines(
                  "#{"#{failure_number}." if failure_number}#{index + 1}",
                  colorizer
                )
              end
            end
          end

          module EmptyBacktraceFormatter
            def self.format_backtrace(*)
              []
            end
          end

          class CommonBacktraceTruncater
            def initialize(parent)
              @parent = parent
            end

            def with_truncated_backtrace(child)
              child_bt  = child.backtrace
              parent_bt = @parent.backtrace
              return child if child_bt.nil? || child_bt.empty? || parent_bt.nil?

              index_before_first_common_frame = -1.downto(-child_bt.size).find do |index|
                parent_bt[index] != child_bt[index]
              end

              return child if index_before_first_common_frame.nil?
              return child if index_before_first_common_frame == -1

              child = child.dup
              child.set_backtrace(child_bt[0..index_before_first_common_frame])
              child
            end
          end
        end

        PENDING_DETAIL_FORMATTER = Proc.new do |example, colorizer|
          colorizer.wrap("# #{example.execution_result.pending_message}", :detail)
        end
      end
    end

    class MultipleExceptionError < StandardError
      module InterfaceTag
        def add(exception)
          return if Pending::PendingExampleFixedError === exception

          return if exception == self

          all_exceptions << exception

          if exception.class.name =~ ::Rex::Core::Formatters::ExceptionPresenter::RSPEC_CLASS_NAME
            failures << exception
          else
            other_errors << exception
          end
        end

        def self.for(ex)
          return ex if self === ex
          MultipleExceptionError.new(ex)
        end
      end

      include InterfaceTag

      attr_reader :failures

      attr_reader :other_errors

      attr_reader :all_exceptions

      attr_reader :aggregation_metadata

      attr_reader :aggregation_block_label

      def initialize(*exceptions)
        super()

        @failures                = []
        @other_errors            = []
        @all_exceptions          = []
        @aggregation_metadata    = { :hide_backtrace => true }
        @aggregation_block_label = nil

        exceptions.each { |e| add e }
      end

      def message
        all_exceptions.map(&:message).join("\n\n")
      end

      def summary
        "Got #{exception_count_description}"
      end

      def exception_count_description
        failure_count = Formatters::Helpers.pluralize(failures.size, "failure")
        return failure_count if other_errors.empty?
        error_count = Formatters::Helpers.pluralize(other_errors.size, "other error")
        "#{failure_count} and #{error_count}"
      end
    end
  end
end
