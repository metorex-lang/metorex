module Rex
  module Core
    class Runner
      attr_reader :options, :configuration, :world

      def self.autorun
        if autorun_disabled?
          Rex.deprecate("Requiring `rspec/autorun` when running RSpec via the `rspec` command")
          return
        elsif installed_at_exit? || running_in_drb?
          return
        end

        at_exit { perform_at_exit }
        @installed_at_exit = true
      end

      def self.perform_at_exit
        return unless $!.nil? || $!.is_a?(SystemExit)

        invoke
      end

      def self.invoke
        disable_autorun!
        status = run(ARGV, $stderr, $stdout).to_i
        exit(status) if status != 0
      end

      def self.run(args, err=$stderr, out=$stdout)
        trap_interrupt
        options = ConfigurationOptions.new(args)

        if options.options[:runner]
          options.options[:runner].call(options, err, out)
        else
          new(options).run(err, out)
        end
      end

      def initialize(options, configuration=Rex.configuration, world=Rex.world)
        @options       = options
        @configuration = configuration
        @world         = world
      end

      def run(err, out)
        setup(err, out)
        return @configuration.reporter.exit_early(exit_code) if Rex.world.wants_to_quit

        run_specs(@world.ordered_example_groups).tap do
          persist_example_statuses
        end
      end

      def setup(err, out)
        configure(err, out)
        return if Rex.world.wants_to_quit

        load_spec_helper
        return if Rex.world.wants_to_quit

        @configuration.load_spec_files
      ensure
        @world.announce_filters
      end

      # Rex loads spec_helper.rb first whenever it is there, unless a
      # `--require` already loaded it.
      def load_spec_helper
        helper = File.expand_path(File.join(@configuration.default_path, "spec_helper.rb"))
        return unless File.file?(helper)
        return if $LOADED_FEATURES.include?(helper)

        @configuration.__send__(:load_file_handling_errors, :require, helper)
      end

      def run_specs(example_groups)
        examples_count = @world.example_count(example_groups)
        examples_passed = @configuration.reporter.report(examples_count) do |reporter|
          @configuration.with_suite_hooks do
            if examples_count == 0 && @configuration.fail_if_no_examples
              return @configuration.failure_exit_code
            end

            if parallel?(example_groups)
              !Parallel.run(example_groups, @configuration.jobs, reporter) && reporter.failed_examples.empty?
            else
              example_groups.map { |g| g.run(reporter) }.all?
            end
          end
        end

        Parallel.verify(example_groups, @world) if parallel?(example_groups) && @configuration.verify_parallel
        examples_passed = report_coverage && examples_passed if @configuration.coverage
        exit_code(examples_passed)
      end

      # Workers run the groups when there are jobs to share them between and
      # the run is a real one.
      def parallel?(example_groups)
        @configuration.jobs > 1 && example_groups.size > 1 && !@configuration.dry_run
      end

      # The counts of this process and of each worker, reported together.
      # Answers whether coverage met the threshold.
      def report_coverage
        data = CoverageReport.collect
        Parallel.worker_coverage.each { |counts| CoverageReport.merge(data, counts) }
        CoverageReport.report(data, @configuration)
      end

      def configure(err, out)
        @configuration.error_stream = err
        @configuration.output_stream = out if @configuration.output_stream == $stdout
        # Measuring starts before anything the suite requires is loaded,
        # since only a file loaded while it is on is counted.
        CoverageReport.start if @options.options[:coverage]
        @options.configure(@configuration)
      end

      def self.disable_autorun!
        @autorun_disabled = true
      end

      def self.autorun_disabled?
        @autorun_disabled ||= false
      end

      def self.installed_at_exit?
        @installed_at_exit ||= false
      end

      def self.running_in_drb?
        return false unless defined?(DRb)

        server = begin
                   DRb.current_server
                 rescue DRb::DRbServerNotFound
                   return false
                 end

        return false unless server && server.alive?

        require 'socket'
        require 'uri'

        local_ipv4 = begin
                       IPSocket.getaddress(Socket.gethostname)
                     rescue SocketError
                       return false
                     end

        ["127.0.0.1", "localhost", local_ipv4].any? { |addr| addr == URI(DRb.current_server.uri).host }
      end

      def self.trap_interrupt
        trap('INT') { handle_interrupt }
      end

      def self.handle_interrupt
        if Rex.world.wants_to_quit
          exit!(1)
        else
          Rex.world.wants_to_quit = true

          $stderr.puts(
            "\nRSpec is shutting down and will print the summary report... Interrupt again to force quit " \
            "(warning: at_exit hooks will be skipped if you force quit)."
          )
        end
      end

      def exit_code(examples_passed=false)
        return @configuration.error_exit_code || @configuration.failure_exit_code if @world.non_example_failure
        return @configuration.failure_exit_code unless examples_passed

        0
      end

    private

      def persist_example_statuses
        return if @configuration.dry_run
        return unless (path = @configuration.example_status_persistence_file_path)

        ExampleStatusPersister.persist(@world.all_examples, path)
      rescue SystemCallError => e
        Rex.warning "Could not write example statuses to #{path} (configured as " \
                      "`config.example_status_persistence_file_path`) due to a " \
                      "system error: #{e.inspect}. Please check that the config " \
                      "option is set to an accessible, valid file path", :call_site => nil
      end
    end
  end
end
