require "set"

module Rex
  module Core
    DeprecationError = Class.new(StandardError)

    # The first frame of the caller's stack outside Rex, as a deprecation
    # names where it was called from.
    def self.first_non_rex_line
      caller_locations.find { |location| !Rex::Core.framework_location?(location) && !location.path.to_s.start_with?("<internal:") }&.to_s
    end

    module Formatters
      DeprecationNotification = Struct.new(:deprecated, :message, :replacement, :call_site) do
        def self.from_hash(data)
          new(data[:deprecated], data[:message], data[:replacement], data[:call_site])
        end
      end

      # Collects deprecations and prints them after the run, or one at a
      # time to a file, or raises each when
      # `config.raise_errors_for_deprecations!` is on.
      class DeprecationFormatter
        Formatters.register self, :deprecation, :deprecation_summary

        attr_reader :count, :deprecation_stream, :summary_stream
        alias output deprecation_stream

        def initialize(deprecation_stream, summary_stream)
          @deprecation_stream = deprecation_stream
          @summary_stream = summary_stream
          @seen_deprecations = Set.new
          @count = 0
        end

        def retarget(deprecation_stream)
          return unless @count.zero?

          @deprecation_stream = deprecation_stream
          @printer = nil
        end

        def printer
          @printer ||= case deprecation_stream
                       when File
                         ImmediatePrinter.new(FileStream.new(deprecation_stream), summary_stream, self)
                       when RaiseErrorStream
                         ImmediatePrinter.new(deprecation_stream, summary_stream, self)
                       else
                         DelayedPrinter.new(deprecation_stream, summary_stream, self)
                       end
        end

        def deprecation(notification)
          return if @seen_deprecations.include?(notification)

          @count += 1
          printer.print_deprecation_message(notification)
          @seen_deprecations << notification
        end

        def deprecation_summary(_notification)
          printer.deprecation_summary
        end

        def deprecation_message_for(data)
          data.message ? SpecifiedDeprecationMessage.new(data) : GeneratedDeprecationMessage.new(data)
        end

        RAISE_ERROR_CONFIG_NOTICE = <<~TEXT

          If you need more of the backtrace for any of these deprecations to
          identify where to make the necessary changes, you can configure
          `config.raise_errors_for_deprecations!`, and it will turn the
          deprecation warnings into errors, giving you the full backtrace.
        TEXT

        DEPRECATION_STREAM_NOTICE = "Pass `--deprecation-out` or set " \
                                    "`config.deprecation_stream` to a file for full output."
        TOO_MANY_WARNINGS_NOTICE = "Too many similar deprecation messages " \
                                   "reported, disregarding further reports. #{DEPRECATION_STREAM_NOTICE}"

        SpecifiedDeprecationMessage = Struct.new(:type) do
          def initialize(data)
            @message = data.message
            super(data.message.gsub(%r{(\w+/)+\w+\.rb:\d+}, ""))
          end

          def to_s
            return @message unless @message.lines.count > 1

            separator = "-" * 80
            "#{separator}\n#{@message.chomp}\n#{separator}"
          end

          def too_many_warnings_message
            TOO_MANY_WARNINGS_NOTICE
          end
        end

        GeneratedDeprecationMessage = Struct.new(:type) do
          def initialize(data)
            @data = data
            super(data.deprecated)
          end

          def to_s
            message = +"#{@data.deprecated} is deprecated."
            message << " Use #{@data.replacement} instead." if @data.replacement
            message << " Called from #{@data.call_site}." if @data.call_site
            message
          end

          def too_many_warnings_message
            "Too many uses of deprecated '#{type}'. #{DEPRECATION_STREAM_NOTICE}"
          end
        end

        class ImmediatePrinter
          attr_reader :deprecation_stream, :summary_stream, :deprecation_formatter

          def initialize(deprecation_stream, summary_stream, deprecation_formatter)
            @deprecation_stream = deprecation_stream
            @summary_stream = summary_stream
            @deprecation_formatter = deprecation_formatter
          end

          def print_deprecation_message(data)
            deprecation_stream.puts deprecation_formatter.deprecation_message_for(data).to_s
          end

          def deprecation_summary
            return if deprecation_formatter.count.zero?

            deprecation_stream.summarize(summary_stream, deprecation_formatter.count)
          end
        end

        # Prints each kind of deprecation up to three times, after the run.
        class DelayedPrinter
          TOO_MANY_USES_LIMIT = 4

          attr_reader :deprecation_stream, :summary_stream, :deprecation_formatter

          def initialize(deprecation_stream, summary_stream, deprecation_formatter)
            @deprecation_stream = deprecation_stream
            @summary_stream = summary_stream
            @deprecation_formatter = deprecation_formatter
            @seen_deprecations = Hash.new { 0 }
            @deprecation_messages = Hash.new { |hash, key| hash[key] = [] }
          end

          def print_deprecation_message(data)
            deprecation_message = deprecation_formatter.deprecation_message_for(data)
            @seen_deprecations[deprecation_message] += 1
            stash_deprecation_message(deprecation_message)
          end

          def stash_deprecation_message(deprecation_message)
            if @seen_deprecations[deprecation_message] < TOO_MANY_USES_LIMIT
              @deprecation_messages[deprecation_message] << deprecation_message.to_s
            elsif @seen_deprecations[deprecation_message] == TOO_MANY_USES_LIMIT
              @deprecation_messages[deprecation_message] << deprecation_message.too_many_warnings_message
            end
          end

          def deprecation_summary
            return unless @deprecation_messages.any?

            print_deferred_deprecation_warnings
            deprecation_stream.puts RAISE_ERROR_CONFIG_NOTICE
            summary_stream.puts "\n#{Formatters.pluralize(deprecation_formatter.count, 'deprecation warning')} total"
          end

          def print_deferred_deprecation_warnings
            deprecation_stream.puts "\nDeprecation Warnings:\n\n"
            @deprecation_messages.keys.sort_by(&:type).each do |deprecation|
              @deprecation_messages[deprecation].each { |message| deprecation_stream.puts message }
              deprecation_stream.puts
            end
          end
        end

        class RaiseErrorStream
          def puts(message)
            raise DeprecationError, message
          end

          def summarize(summary_stream, deprecation_count)
            summary_stream.puts "\n#{Formatters.pluralize(deprecation_count, 'deprecation')} found."
          end
        end

        class FileStream
          def initialize(file)
            @file = file
            @file.sync = true
          end

          def puts(*arguments)
            @file.puts(*arguments)
          end

          def summarize(summary_stream, deprecation_count)
            path = @file.respond_to?(:path) ? @file.path : @file.inspect
            summary_stream.puts "\n#{Formatters.pluralize(deprecation_count, 'deprecation')} logged to #{path}"
            puts RAISE_ERROR_CONFIG_NOTICE
          end
        end
      end
    end

    class Reporter
      def deprecation(hash)
        notify(:deprecation, Formatters::DeprecationNotification.from_hash(hash))
      end
    end

    class Configuration
      def raise_errors_for_deprecations!
        self.deprecation_stream = Formatters::DeprecationFormatter::RaiseErrorStream.new
      end

      # A deprecation reported before the run's reporter is set up waits
      # until it is.
      def report_deprecation(data)
        reporter = Rex.world.reporter
        if reporter&.set_up?
          reporter.deprecation(data)
        else
          pending_deprecations << data
        end
      end

      def pending_deprecations
        @pending_deprecations ||= []
      end
    end
  end

  def self.deprecate(deprecated, data = {})
    Rex.configuration.report_deprecation({ deprecated: deprecated, call_site: Core.first_non_rex_line }.merge(data))
  end

  def self.warn_deprecation(message, options = {})
    Rex.configuration.report_deprecation(options.merge(message: message))
  end
end
