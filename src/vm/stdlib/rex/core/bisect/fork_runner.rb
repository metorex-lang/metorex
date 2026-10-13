require 'stringio'
require "rex/core/formatters/base_bisect_formatter"
require "rex/core/bisect/utilities"

module Rex
  module Core
    module Bisect
      class ForkRunner
        def self.start(shell_command, spec_runner)
          instance = new(shell_command, spec_runner)
          yield instance
        ensure
          instance.shutdown
        end

        def self.name
          :fork
        end

        def initialize(shell_command, spec_runner)
          @shell_command = shell_command
          @channel = Channel.new
          @run_dispatcher = RunDispatcher.new(spec_runner, @channel)
        end

        def run(locations)
          run_descriptor = ExampleSetDescriptor.new(locations, original_results.failed_example_ids)
          dispatch_run(run_descriptor)
        end

        def original_results
          @original_results ||= dispatch_run(ExampleSetDescriptor.new(
            @shell_command.original_locations, []))
        end

        def shutdown
          @channel.close
        end

      private

        def dispatch_run(run_descriptor)
          @run_dispatcher.dispatch_specs(run_descriptor)
          @channel.receive.tap do |result|
            if result.is_a?(String)
              raise BisectFailedError.for_failed_spec_run(result)
            end
          end
        end

        class RunDispatcher
          def initialize(runner, channel)
            @runner = runner
            @channel = channel

            @spec_output = StringIO.new

            runner.configuration.tap do |c|
              c.reset_reporter
              c.output_stream = @spec_output
              c.error_stream = @spec_output
            end
          end

          def dispatch_specs(run_descriptor)
            pid = fork { run_specs(run_descriptor) }
            Process.detach(pid)
          end

        private

          def run_specs(run_descriptor)
            $stdout = $stderr = @spec_output
            formatter = CaptureFormatter.new(run_descriptor.failed_example_ids)

            @runner.configuration.tap do |c|
              c.files_or_directories_to_run = run_descriptor.all_example_ids
              c.formatter = formatter
              c.load_spec_files
            end

            @runner.world.announce_filters

            @runner.run_specs(@runner.world.ordered_example_groups)
            latest_run_results = formatter.results

            if latest_run_results.nil? || latest_run_results.all_example_ids.empty?
              @channel.send(@spec_output.string)
            else
              @channel.send(latest_run_results)
            end
          end
        end

        class CaptureFormatter < Formatters::BaseBisectFormatter
          attr_accessor :results
          alias_method :notify_results, :results=
        end
      end
    end
  end
end
