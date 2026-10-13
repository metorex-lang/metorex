require "rex/core/formatters/base_formatter"

module Rex
  module Core
    module Formatters
      class FailureListFormatter < BaseFormatter
        Formatters.register self, :example_failed, :dump_profile, :message

        def example_failed(failure)
          output.puts "#{failure.example.location}:#{failure.example.description}"
        end

        def dump_profile(_profile); end
        def message(_message); end
      end
    end
  end
end
