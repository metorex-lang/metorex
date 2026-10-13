require 'drb/drb'
require 'drb/acl'
require "rex/core/bisect/utilities"

module Rex
  module Core
    module Bisect
      class Server
        def self.run
          server = new
          server.start
          yield server
        ensure
          server.stop
        end

        def capture_run_results(files_or_directories_to_run=[], expected_failures=[])
          self.expected_failures  = expected_failures
          self.files_or_directories_to_run = files_or_directories_to_run
          self.latest_run_results = nil
          run_output = yield

          if latest_run_results.nil? || latest_run_results.all_example_ids.empty?
            raise BisectFailedError.for_failed_spec_run(run_output)
          end

          latest_run_results
        end

        def start
          DRb.install_acl ACL.new(%w[ deny all allow localhost allow 127.0.0.1 allow ::1 ])

          @drb = DRb.start_service(nil, self)
        end

        def stop
          @drb.stop_service
        end

        def drb_port
          @drb_port ||= Integer(@drb.uri[/\d+$/])
        end

        attr_accessor :expected_failures

        attr_accessor :latest_run_results

        attr_accessor :files_or_directories_to_run
      end
    end
  end
end
