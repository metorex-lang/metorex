require 'rake'
require 'rake/tasklib'
require 'rspec/support'

unless Rex::Support.respond_to?(:require_rspec_core)
  Rex::Support.define_optimized_require_for_rspec(:core) { |f| require_relative "../#{f}" }
end

require "rex/core/shell_escape"

module Rex
  module Core
    class RakeTask < ::Rake::TaskLib
      include ::Rake::DSL if defined?(::Rake::DSL)
      include Rex::Core::ShellEscape

      DEFAULT_RSPEC_PATH = File.expand_path('../../../../exe/rspec', __FILE__)

      DEFAULT_PATTERN = 'spec/**{,/*/**}/*_spec.rb'

      attr_accessor :name

      attr_accessor :pattern

      attr_accessor :exclude_pattern

      attr_accessor :fail_on_error

      attr_accessor :failure_message

      if RUBY_VERSION < "1.9.0" || Support::Ruby.jruby?
        def with_clean_environment=(_value)
          raise ArgumentError, "Running in a clean environment is not supported on Ruby versions before 1.9.0"
        end

        def with_clean_environment
          false
        end
      else
        attr_accessor :with_clean_environment
      end

      attr_accessor :verbose

      attr_accessor :ruby_opts

      attr_accessor :rspec_path

      attr_accessor :rspec_opts

      def initialize(*args, &task_block)
        @name          = args.shift || :spec
        @ruby_opts     = nil
        @rspec_opts    = nil
        @verbose       = true
        @fail_on_error = true
        @rspec_path    = DEFAULT_RSPEC_PATH
        @pattern       = DEFAULT_PATTERN

        define(args, &task_block)
      end

      def run_task(verbose)
        command = spec_command
        puts command if verbose

        if with_clean_environment
          return if system({}, command, :unsetenv_others => true)
        else
          return if system(command)
        end

        puts failure_message if failure_message

        return unless fail_on_error
        $stderr.puts "#{command} failed" if verbose
        exit $?.exitstatus || 1
      end

    private

      def define(args, &task_block)
        desc "Run RSpec code examples" unless ::Rake.application.last_description

        task name, *args do |_, task_args|
          RakeFileUtils.__send__(:verbose, verbose) do
            task_block.call(*[self, task_args].slice(0, task_block.arity)) if task_block
            run_task verbose
          end
        end
      end

      def file_inclusion_specification
        if ENV['SPEC']
          FileList[ENV['SPEC']].sort
        elsif String === pattern && !File.exist?(pattern)
          return if [*rspec_opts].any? { |opt| opt =~ /--pattern/ }
          "--pattern #{escape pattern}"
        else
          FileList[pattern].sort.map { |file| escape file }
        end
      end

      def file_exclusion_specification
        " --exclude-pattern #{escape exclude_pattern}" if exclude_pattern
      end

      def spec_command
        cmd_parts = []
        cmd_parts << RUBY
        cmd_parts << ruby_opts
        cmd_parts << rspec_load_path
        cmd_parts << escape(rspec_path)
        cmd_parts << file_inclusion_specification
        cmd_parts << file_exclusion_specification
        cmd_parts << rspec_opts
        cmd_parts.flatten.reject(&blank).join(" ")
      end

      def blank
        lambda { |s| s.nil? || s == "" }
      end

      def rspec_load_path
        @rspec_load_path ||= begin
          core_and_support = $LOAD_PATH.grep(
            /#{File::SEPARATOR}rspec-(core|support)[^#{File::SEPARATOR}]*#{File::SEPARATOR}lib/
          ).uniq

          "-I#{core_and_support.map { |file| escape file }.join(File::PATH_SEPARATOR)}"
        end
      end
    end
  end
end
