module Rex
  module Core
    # Raised by `skip` inside an example to stop it there.
    class SkipDeclaredInExample < StandardError
      attr_reader :argument

      def initialize(argument)
        @argument = argument
        super(argument.to_s)
      end
    end

    # Raised for an example marked pending that passed.
    class PendingExampleFixedError < StandardError
    end

    module Pending
      NO_REASON_GIVEN = "No reason given"
      NOT_YET_IMPLEMENTED = "Not yet implemented"
      SkipDeclaredInExample = Core::SkipDeclaredInExample
      PendingExampleFixedError = Core::PendingExampleFixedError

      # A pending example that passed.
      def self.mark_fixed!(example)
        example.execution_result.pending_fixed = true
      end

      def pending(message = nil, &block)
        current_example = Rex.current_example
        if block
          raise ArgumentError, "Passing a block within an example is now deprecated. Use `skip` instead."
        elsif current_example
          Pending.mark_pending!(current_example, message)
        else
          raise "`pending` may not be used outside of examples, such as in before(:context). Maybe you want `skip`?"
        end
      end

      def skip(message = nil)
        current_example = Rex.current_example
        Pending.mark_skipped!(current_example, message) if current_example
        raise SkipDeclaredInExample.new(message)
      end

      def self.mark_pending!(example, message)
        message = NO_REASON_GIVEN if message.nil? || message == true
        example.metadata[:pending] = true
        example.execution_result.pending_message = message.to_s
        example.execution_result.pending_fixed = false
      end

      def self.mark_skipped!(example, message)
        mark_pending!(example, message)
        example.metadata[:skip] = true
      end
    end

    # What happened when an example ran.
    class ExecutionResult
      attr_accessor :status, :exception, :pending_message, :pending_exception,
                    :pending_fixed, :started_at, :finished_at, :run_time

      def pending_fixed?
        @pending_fixed ? true : false
      end

      def example_skipped?
        status == :pending && pending_exception.nil?
      end
    end

    class Example
      attr_reader :example_group, :metadata, :execution_result, :example_group_instance
      attr_accessor :exception

      def initialize(example_group, description, user_metadata, block, location)
        @example_group = example_group
        @block = block
        @execution_result = ExecutionResult.new
        file_path, line_number = location
        @metadata = example_group.metadata.reject { |key, _| ExampleGroup::GROUP_ONLY_KEYS.include?(key) }
        @metadata.update(user_metadata)
        @metadata[:example_group] = example_group.metadata
        @metadata[:description_args] = description.nil? ? [] : [description]
        @metadata[:description] = description.to_s
        @metadata[:file_path] = file_path
        @metadata[:line_number] = line_number
        @metadata[:location] = "#{file_path}:#{line_number}"
        @metadata[:absolute_file_path] = File.expand_path(file_path)
        @metadata[:block] = block
        @metadata[:rerun_file_path] = example_group.metadata[:rerun_file_path] || file_path
        @metadata[:shared_group_inclusion_backtrace] = SharedGroupInclusion.current_backtrace
        @metadata[:skip] = "Not yet implemented" if block.nil? && !@metadata.key?(:skip)
        @metadata[:execution_result] = @execution_result
        refresh_full_description
        Rex.configuration.derived_metadata_blocks.dup.each do |filter, derive|
          derive.call(@metadata) if Metadata.filter_applies?(filter, @metadata)
        end
      end

      def refresh_full_description
        @metadata[:full_description] = ExampleGroup.build_description_from(
          example_group.metadata[:full_description], @metadata[:description]
        )
      end

      def description
        description = @metadata[:description].to_s
        description = "example at #{location}" if description.empty?
        Rex.configuration.format_docstrings_block.call(description)
      end

      def full_description
        @metadata[:full_description]
      end

      def location
        @metadata[:location]
      end

      def file_path
        @metadata[:file_path]
      end

      def line_number
        @metadata[:line_number]
      end

      def id
        "#{@metadata[:rerun_file_path]}[#{scoped_id}]"
      end

      # `./spec/a_spec.rb[1:2]` as the file and the scoped id.
      def self.parse_id(id)
        id.match(/\A(.*?)(?:\[([\d\s:,]+)\])?\z/).captures
      end

      def scoped_id
        @metadata[:scoped_id]
      end

      def rerun_argument
        location_rerun_argument
      end

      # The location of this example, or of the nearest group around it, in
      # a file that was loaded as a spec file. An example a shared group
      # defined is located in the support file, which a rerun cannot name.
      def location_rerun_argument
        @location_rerun_argument ||= begin
          loaded = Rex.configuration.loaded_spec_files
          found = nil
          meta = @metadata
          while meta && found.nil?
            found = meta[:location] if loaded.include?(meta[:absolute_file_path])
            meta = meta[:example_group] || meta[:parent_example_group]
          end
          found || location
        end
      end

      def pending?
        @metadata[:pending] ? true : false
      end

      def skipped?
        @metadata[:skip] ? true : false
      end

      def inspect
        "#<#{self.class.name} #{description.inspect}>"
      end
      alias to_s inspect

      # Runs the example in a fresh instance of its group and reports it.
      def run(example_group_instance, reporter)
        @example_group_instance = example_group_instance
        Rex.current_example = self
        @execution_result.started_at = Rex::Core.monotonic_now
        reporter.example_started(self)

        if skipped?
          Pending.mark_skipped!(self, @metadata[:skip])
        elsif Rex.configuration.dry_run
          # A dry run builds every group and runs no example body.
        else
          if @metadata[:pending] && !@execution_result.pending_message
            Pending.mark_pending!(self, @metadata[:pending])
          end
          Rex::Mocks.setup
          run_with_around_hooks do
            begin
              run_before_hooks
              if @metadata[:aggregate_failures]
                example = self
                Rex::Expectations::FailureAggregator.new(nil, hide_backtrace: true).aggregate do
                  @example_group_instance.instance_exec(example, &@block)
                end
              else
                @example_group_instance.instance_exec(self, &@block)
              end
              if pending?
                @execution_result.pending_fixed = true
                raise PendingExampleFixedError, "Expected pending '#{@execution_result.pending_message}' to fail. No error was raised."
              end
            rescue SkipDeclaredInExample
              # The example already marked itself skipped.
            rescue *Rex::Core::DANGEROUS_EXCEPTIONS
              raise
            rescue Exception => error
              record_exception(error)
            ensure
              run_after_hooks
            end
          end
          verify_and_take_down_mocks
        end

        assign_generated_description
        finish(reporter)
      ensure
        Rex.current_example = nil
        @example_group_instance = nil
      end

      # Expectations on doubles are checked once the example and its `after`
      # hooks are done, and every stub is taken down whatever happened.
      def verify_and_take_down_mocks
        Rex::Mocks.verify if @exception.nil? || pending?
      rescue *Rex::Core::DANGEROUS_EXCEPTIONS
        raise
      rescue Exception => error
        record_exception(error)
      ensure
        Rex::Mocks.teardown
      end

      # Forgets the outcome of a run, so the example can run again.
      def reset_for_rerun
        @exception = nil
        @execution_result = ExecutionResult.new
        @metadata[:execution_result] = @execution_result
      end

      def record_exception(error)
        return @exception = error unless @exception

        @exception = MultipleExceptionError.new(@exception) unless MultipleExceptionError === @exception
        @exception.add(error)
      end

      def assign_generated_description
        if @metadata[:description].to_s.empty? && (generated = Rex::Matchers.generated_description)
          @metadata[:description] = generated
          refresh_full_description
        end
        Rex::Matchers.clear_generated_description
      end

      def finish(reporter)
        @execution_result.finished_at = Rex::Core.monotonic_now
        @execution_result.run_time = @execution_result.finished_at - @execution_result.started_at
        if pending? && !@execution_result.pending_fixed?
          @execution_result.status = :pending
          @execution_result.pending_exception = @exception
          @exception = nil
          reporter.example_finished(self)
          reporter.example_pending(self)
          true
        elsif @exception
          @execution_result.status = :failed
          @execution_result.exception = @exception
          reporter.example_finished(self)
          reporter.example_failed(self)
          false
        else
          @execution_result.status = :passed
          reporter.example_finished(self)
          reporter.example_passed(self)
          true
        end
      end

      # Marks the example failed without running it, the way an error in a
      # `before(:context)` hook fails every example in its group.
      def fail_with_exception(reporter, error)
        Rex.current_example = self
        @execution_result.started_at = Rex::Core.monotonic_now
        reporter.example_started(self)
        @exception = error
        finish(reporter)
      ensure
        Rex.current_example = nil
      end

      def skip_with_exception(reporter, error)
        fail_with_exception(reporter, error)
      end

      def run_before_hooks
        @example_group_instance.__send__(:__rex_install_context_ivars)
        hooks_for(:before).each do |hook|
          @example_group_instance.instance_exec(self, &hook.block)
        end
      end

      def run_after_hooks
        hooks_for(:after).each do |hook|
          begin
            @example_group_instance.instance_exec(self, &hook.block)
          rescue *Rex::Core::DANGEROUS_EXCEPTIONS
            raise
          rescue SkipDeclaredInExample
          rescue Exception => error
            record_exception(error)
          end
        end
      end

      # `before` hooks run from the configuration inward through each group,
      # and `after` hooks run the other way.
      def hooks_for(kind)
        chain = example_group.parent_groups.reverse
        hooks = []
        if kind == :before
          hooks.concat(applicable(Rex.configuration.hooks.for(:before, :example)))
          chain.each { |group| hooks.concat(applicable(group.hooks.for(:before, :example))) }
        else
          chain.reverse_each { |group| hooks.concat(applicable(group.hooks.for(:after, :example))) }
          hooks.concat(applicable(Rex.configuration.hooks.for(:after, :example)))
        end
        hooks
      end

      def applicable(hooks)
        hooks.select { |hook| hook.applies_to?(@metadata) }
      end

      def run_with_around_hooks(&inner)
        chain = example_group.parent_groups.reverse
        arounds = applicable(Rex.configuration.hooks.for(:around, :example))
        chain.each { |group| arounds.concat(applicable(group.hooks.for(:around, :example))) }
        runner = inner
        arounds.reverse_each do |hook|
          wrapped = runner
          runner = lambda do
            procsy = Procsy.new(self, &wrapped)
            begin
              @example_group_instance.instance_exec(procsy, &hook.block)
            rescue *Rex::Core::DANGEROUS_EXCEPTIONS
              raise
            rescue SkipDeclaredInExample
            rescue Exception => error
              record_exception(error)
            end
          end
        end
        runner.call
      end

      # What an `around` hook receives: the example, which it runs with `run`
      # or `call`.
      class Procsy
        attr_reader :example

        def initialize(example, &block)
          @example = example
          @proc = block
          @executed = false
        end

        def call(*arguments, &block)
          @executed = true
          @proc.call(*arguments, &block)
        end
        alias run call

        def to_proc
          method(:call).to_proc
        end

        def executed?
          @executed
        end

        def metadata
          @example.metadata
        end

        def description
          @example.description
        end

        def full_description
          @example.full_description
        end

        def location
          @example.location
        end

        def exception
          @example.exception
        end

        def pending?
          @example.pending?
        end

        def skipped?
          @example.skipped?
        end

        def example_group
          @example.example_group
        end

        def inspect
          @example.inspect.sub("Example", "ExampleProcsy")
        end
      end
    end

    # Several exceptions from one example, such as a failure in the body and
    # another in an `after` hook. Rex's own failures count as failures, and
    # everything else as other errors.
    class MultipleExceptionError < StandardError
      attr_reader :failures, :other_errors, :all_exceptions, :aggregation_metadata, :aggregation_block_label

      def initialize(*exceptions)
        super()
        @failures = []
        @other_errors = []
        @all_exceptions = []
        @aggregation_metadata = { hide_backtrace: true }
        @aggregation_block_label = nil
        exceptions.each { |error| add(error) }
      end

      def add(error)
        return if error.is_a?(PendingExampleFixedError)
        return if error.equal?(self)

        all_exceptions << error
        if error.class.name.to_s =~ /RSpec|\ARex::/
          failures << error
        else
          other_errors << error
        end
      end

      def message
        all_exceptions.map(&:message).join("\n\n")
      end

      def summary
        "Got #{exception_count_description}"
      end

      def exception_count_description
        failure_count = Formatters.pluralize(failures.size, "failure")
        return failure_count if other_errors.empty?

        "#{failure_count} and #{Formatters.pluralize(other_errors.size, 'other error')}"
      end
    end

    DANGEROUS_EXCEPTIONS = [NoMemoryError, SignalException, Interrupt, SystemExit].freeze

    def self.monotonic_now
      Process.clock_gettime(Process::CLOCK_MONOTONIC)
    end
  end

  def self.current_example
    Thread.current[:rex_current_example]
  end

  def self.current_example=(example)
    Thread.current[:rex_current_example] = example
  end
end
