require 'drb/drb'
require "rex/core/formatters/base_bisect_formatter"

module Rex
  module Core
    module Formatters
      class BisectDRbFormatter < BaseBisectFormatter
        def initialize(_output)
          drb_uri = "druby://localhost:#{Rex.configuration.drb_port}"
          @bisect_server = DRbObject.new_with_uri(drb_uri)
          Rex.configuration.files_or_directories_to_run = @bisect_server.files_or_directories_to_run
          super(Set.new(@bisect_server.expected_failures))
        end

        def notify_results(results)
          @bisect_server.latest_run_results = results
        end
      end
    end
  end
end
