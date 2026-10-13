module Rex
  module Core
    # The settings Rex has that RSpec does not.
    class Configuration
      def diff_enabled?
        value = value_for(:diff_enabled) { @diff_enabled }
        value.nil? ? true : value
      end

      attr_writer :diff_enabled

      # The most lines either side of a comparison may have for a failure to
      # print a diff of them. None by default, so every diff prints.
      attr_accessor :diff_line_limit

      # Whether a failure names the expression handed to `expect`, as in
      # `expected user.age to eq 42, got 41`. Off by default, so failures
      # read as RSpec writes them.
      attr_accessor :source_expressions

      # How many lines around the failing statement a failure prints. None
      # by default, so a failure quotes only the statement, as RSpec does.
      attr_writer :failure_context_lines

      def failure_context_lines
        @failure_context_lines || 0
      end

      # `--coverage` and the settings that shape its report.
      attr_writer :coverage, :coverage_threshold, :coverage_dir, :coverage_formats, :coverage_exclude

      def coverage
        value_for(:coverage) { @coverage }
      end

      def coverage_threshold
        value_for(:coverage_threshold) { @coverage_threshold }
      end

      def coverage_dir
        value_for(:coverage_dir) { @coverage_dir } || "coverage"
      end

      def coverage_formats
        value_for(:coverage_formats) { @coverage_formats } || ["summary"]
      end

      def coverage_exclude
        value_for(:coverage_exclude) { @coverage_exclude ||= [] }
      end

      # Whether the native versions of the hot parts of Rex run. Off, the
      # Ruby versions run, which a divergence can be checked against.
      def fast_paths
        value = value_for(:fast_paths) { @fast_paths }
        value.nil? ? true : value
      end
      alias fast_paths? fast_paths

      def fast_paths=(value)
        @fast_paths = value
        __fast_paths__(fast_paths)
      end

      # How many worker processes the top-level groups run in.
      def jobs
        value_for(:jobs) { @jobs } || 1
      end
      attr_writer :jobs

      # Whether a parallel run is followed by a serial one whose outcomes it
      # is compared with.
      attr_writer :verify_parallel

      def verify_parallel
        value_for(:verify_parallel) { @verify_parallel }
      end

      # Whether `RSpec` names Rex. It does from load, and a suite with an
      # `RSpec` of its own sets this to false to take the name back.
      def rspec_constant
        defined?(::RSpec) && ::RSpec.equal?(Rex) ? true : false
      end
      alias rspec_constant? rspec_constant

      def rspec_constant=(value)
        if value
          Object.const_set(:RSpec, Rex) if Object.autoload?(:RSpec) || !defined?(::RSpec)
        elsif rspec_constant
          Object.__send__(:remove_const, :RSpec)
        end
      end
    end
  end
end
