module Rex
  module Core
    module Invocations
      class InitializeProject
        def call(*_args)
          require "rex/core/project_initializer"
          ProjectInitializer.new.run
          0
        end
      end

      class DRbWithFallback
        def call(options, err, out)
          require 'rspec/core/drb'
          begin
            return DRbRunner.new(options).run(err, out)
          rescue DRb::DRbConnError
            err.puts "No DRb server is running. Running in local process instead ..."
          end
          Rex::Core::Runner.new(options).run(err, out)
        end
      end

      class Bisect
        def call(options, err, out)
          require "rex/core/bisect/coordinator"
          runner = Runner.new(options).tap { |r| r.configure(err, out) }
          formatter = bisect_formatter_klass_for(options.options[:bisect]).new(
            out, runner.configuration.bisect_runner
          )

          success = Rex::Core::Bisect::Coordinator.bisect_with(
            runner, options.args, formatter
          )

          runner.exit_code(success)
        end

      private

        def bisect_formatter_klass_for(argument)
          return Formatters::BisectDebugFormatter if argument == "verbose"
          Formatters::BisectProgressFormatter
        end
      end

      class PrintVersion
        def call(_options, _err, out)
          overall_version = Rex::Core::Version::STRING
          unless overall_version =~ /[a-zA-Z]+/
            overall_version = overall_version.split('.').first(2).join('.')
          end

          out.puts "RSpec #{overall_version}"

          [:Core, :Expectations, :Mocks, :Rails, :Support].each do |const_name|
            lib_name = const_name.to_s.downcase
            begin
              require "rspec/#{lib_name}/version"
            rescue LoadError
              nil
            else
              out.puts "  - rspec-#{lib_name} #{Rex.const_get(const_name)::Version::STRING}"
            end
          end

          0
        end
      end

      PrintHelp = Struct.new(:parser, :hidden_options) do
        def call(_options, _err, out)
          out.puts parser.to_s.gsub(/^\s+(#{hidden_options.join('|')})\b.*$\n/, '')
          0
        end
      end
    end
  end
end
