require 'open3'
require "rex/core/bisect/server"

module Rex
  module Core
    module Bisect
      class ShellRunner
        def self.start(shell_command, _spec_runner)
          Server.run do |server|
            yield new(server, shell_command)
          end
        end

        def self.name
          :shell
        end

        def initialize(server, shell_command)
          @server        = server
          @shell_command = shell_command
        end

        def run(locations)
          run_locations(locations, original_results.failed_example_ids)
        end

        def original_results
          @original_results ||= run_locations(@shell_command.original_locations)
        end

      private

        def run_locations(*capture_args)
          @server.capture_run_results(*capture_args) do
            run_command @shell_command.command_for([], @server)
          end
        end

        if Open3.respond_to?(:capture2e) && !Rex::Support::Ruby.jruby?
          def run_command(cmd)
            Open3.capture2e(@shell_command.bisect_environment_hash, cmd).first
          end
        else
          def run_command(cmd)
            out = err = nil

            original_spec_opts = ENV['SPEC_OPTS']
            ENV['SPEC_OPTS'] = @shell_command.spec_opts_without_bisect

            Open3.popen3(cmd) do |_, stdout, stderr|
              out = stdout.read
              err = stderr.read
            end

            "Stdout:\n#{out}\n\nStderr:\n#{err}"
          ensure
            ENV['SPEC_OPTS'] = original_spec_opts
          end
        end
      end
    end
  end
end
