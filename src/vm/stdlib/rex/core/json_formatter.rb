require "json"

module Rex
  module Core
    module Formatters
      # `--format json`: the run as one JSON document, in RSpec's shape.
      class JsonFormatter < BaseFormatter
        Formatters.register self, :message, :dump_summary, :dump_profile, :stop, :seed, :close

        attr_reader :output_hash

        def initialize(output)
          super
          @output_hash = { version: Rex::VERSION }
        end

        def message(notification)
          (@output_hash[:messages] ||= []) << notification.message
        end

        def dump_summary(summary)
          @output_hash[:summary] = {
            duration: summary.duration,
            example_count: summary.example_count,
            failure_count: summary.failure_count,
            pending_count: summary.pending_count,
            errors_outside_of_examples_count: summary.errors_outside_of_examples_count
          }
          @output_hash[:summary_line] = summary.totals_line
        end

        def stop(group_notification)
          @output_hash[:examples] = group_notification.notifications.map do |notification|
            format_example(notification.example).tap do |hash|
              exception = notification.example.exception
              if exception
                hash[:exception] = {
                  class: exception.class.name,
                  message: exception.message,
                  backtrace: notification.formatted_backtrace
                }
              end
            end
          end
        end

        def seed(notification)
          return unless notification.seed_used?

          @output_hash[:seed] = notification.seed
        end

        def close(_notification)
          output.write @output_hash.to_json
        end

        def dump_profile(profile)
          @output_hash[:profile] = {}
          dump_profile_slowest_examples(profile)
          dump_profile_slowest_example_groups(profile)
        end

        def dump_profile_slowest_examples(profile)
          @output_hash[:profile] = {}
          @output_hash[:profile][:examples] = profile.slowest_examples.map do |example|
            format_example(example).tap { |hash| hash[:run_time] = example.execution_result.run_time }
          end
          @output_hash[:profile][:slowest] = profile.slow_duration
          @output_hash[:profile][:total] = profile.duration
        end

        def dump_profile_slowest_example_groups(profile)
          @output_hash[:profile] ||= {}
          @output_hash[:profile][:groups] = profile.slowest_groups.map do |location, hash|
            hash.update(location: location)
          end
        end

        private

        def format_example(example)
          {
            id: example.id,
            description: example.description,
            full_description: example.full_description,
            status: example.execution_result.status.to_s,
            file_path: example.metadata[:file_path],
            line_number: example.metadata[:line_number],
            run_time: example.execution_result.run_time,
            pending_message: example.execution_result.pending_message
          }
        end
      end
    end
  end
end
