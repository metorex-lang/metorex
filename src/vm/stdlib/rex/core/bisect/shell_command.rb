require "rex/core/shell_escape"
require 'shellwords'

module Rex
  module Core
    module Bisect
      class ShellCommand
        attr_reader :original_cli_args

        def initialize(original_cli_args)
          @original_cli_args = original_cli_args.reject { |arg| arg.start_with?("--bisect") }
        end

        def command_for(locations, server)
          parts = []

          parts << RUBY << load_path
          parts << open3_safe_escape(Rex::Core.path_to_executable)

          parts << "--format"   << "bisect-drb"
          parts << "--drb-port" << server.drb_port

          parts.concat(reusable_cli_options)
          parts.concat(locations.map { |l| open3_safe_escape(l) })

          parts.join(" ")
        end

        def repro_command_from(locations)
          parts = []

          parts.concat environment_repro_parts
          parts << "rspec"
          parts.concat Formatters::Helpers.organize_ids(locations)
          parts.concat original_cli_args_without_locations

          parts.join(" ")
        end

        def original_locations
          parsed_original_cli_options.fetch(:files_or_directories_to_run)
        end

        def bisect_environment_hash
          if ENV.key?('SPEC_OPTS')
            { 'SPEC_OPTS' => spec_opts_without_bisect }
          else
            {}
          end
        end

        def spec_opts_without_bisect
          Shellwords.join(
            Shellwords.split(ENV.fetch('SPEC_OPTS', '')).reject do |arg|
              arg =~ /^--bisect/
            end
          )
        end

      private

        include Rex::Core::ShellEscape
        if Rex::Support::Ruby.jruby?
          alias open3_safe_escape quote
        else
          alias open3_safe_escape escape
        end

        def environment_repro_parts
          bisect_environment_hash.map do |k, v|
            %Q(#{k}="#{v}")
          end
        end

        def reusable_cli_options
          @reusable_cli_options ||= begin
            opts = original_cli_args_without_locations

            if (port = parsed_original_cli_options[:drb_port])
              opts -= %W[ --drb-port #{port} ]
            end

            parsed_original_cli_options.fetch(:formatters) { [] }.each do |(name, out)|
              opts -= %W[ --format #{name} -f -f#{name} ]
              opts -= %W[ --out #{out} -o -o#{out} ]
            end

            opts
          end
        end

        def original_cli_args_without_locations
          @original_cli_args_without_locations ||= begin
            files_or_dirs = parsed_original_cli_options.fetch(:files_or_directories_to_run)
            @original_cli_args - files_or_dirs
          end
        end

        def parsed_original_cli_options
          @parsed_original_cli_options ||= Parser.parse(@original_cli_args)
        end

        def load_path
          @load_path ||= "-I#{$LOAD_PATH.map { |p| open3_safe_escape(p) }.join(':')}"
        end

        RUBY = File.join(
          RbConfig::CONFIG['bindir'],
          RbConfig::CONFIG['ruby_install_name'] + RbConfig::CONFIG['EXEEXT']).
          sub(/.*\s.*/m, '"\&"')
      end
    end
  end
end
