module Rex
  module Core
    class Example
      def self.delegate_to_metadata(key)
        define_method(key) { @metadata[key] }
      end

      delegate_to_metadata :execution_result
      delegate_to_metadata :file_path
      delegate_to_metadata :full_description
      delegate_to_metadata :location
      delegate_to_metadata :pending
      delegate_to_metadata :skip

      def description
        description = if metadata[:description].to_s.empty?
                        location_description
                      else
                        metadata[:description]
                      end

        Rex.configuration.format_docstrings_block.call(description)
      end

      def inspect_output
        inspect_output = "\"#{description}\""
        unless metadata[:description].to_s.empty?
          inspect_output += " (#{location})"
        end
        inspect_output
      end

      def location_rerun_argument
        @location_rerun_argument ||= begin
          loaded_spec_files = Rex.configuration.loaded_spec_files

          Metadata.ascending(metadata) do |meta|
            break meta[:location] if loaded_spec_files.include?(meta[:absolute_file_path])
          end
        end
      end

      def rerun_argument
        location_rerun_argument
      end

      def id
        @id ||= Metadata.id_from(metadata)
      end

      def self.parse_id(id)
        id.match(/\A(.*?)(?:\[([\d\s:,]+)\])?\z/).captures
      end

      def duplicate_with(metadata_overrides={})
        new_metadata = metadata.clone.merge(metadata_overrides)

        Rex::Core::Metadata::RESERVED_KEYS.each do |reserved_key|
          new_metadata.delete reserved_key
        end

        Example.new(example_group, description.clone,
                    new_metadata, metadata[:block])
      end

      def update_inherited_metadata(updates)
        metadata.update(updates) do |_key, existing_example_value, _new_inherited_value|
          existing_example_value
        end
      end

      attr_reader :exception

      attr_reader :metadata

      attr_reader :example_group_instance

      attr_accessor :clock

      def initialize(example_group_class, description, user_metadata, example_block=nil)
        @example_group_class = example_group_class
        @example_block       = example_block

        example_group_class.examples << self

        @metadata = Metadata::ExampleHash.create(
          @example_group_class.metadata, user_metadata,
          example_group_class.method(:next_runnable_index_for),
          description, example_block
        )

        config = Rex.configuration
        config.apply_derived_metadata_to(@metadata)

        @metadata[:last_run_status] = config.last_run_statuses[id]

        @example_group_instance = @exception = nil
        @clock = Rex::Core::Time
        @reporter = Rex::Core::NullReporter
      end

      def inspect
        "#<#{self.class.name} #{description.inspect}>"
      end
      alias to_s inspect

      attr_reader :reporter

      def example_group
        @example_group_class
      end

      def pending?
        !!pending
      end

      def skipped?
        !!skip
      end

      def run(example_group_instance, reporter)
        @example_group_instance = example_group_instance
        @reporter = reporter
        Rex.configuration.configure_example(self, hooks)
        Rex.current_example = self

        start(reporter)
        Pending.mark_pending!(self, pending) if pending?

        begin
          if skipped?
            Pending.mark_pending! self, skip
          elsif !Rex.configuration.dry_run?
            with_around_and_singleton_context_hooks do
              begin
                run_before_example
                Rex.current_scope = :example
                @example_group_instance.instance_exec(self, &@example_block)

                if pending?
                  Pending.mark_fixed! self

                  raise Pending::PendingExampleFixedError,
                        'Expected example to fail since it is pending, but it passed.',
                        [location]
                end
              rescue Pending::SkipDeclaredInExample => _
              rescue AllExceptionsExcludingDangerousOnesOnRubiesThatAllowIt => e
                set_exception(e)
              ensure
                Rex.current_scope = :after_example_hook
                run_after_example
              end
            end
          end
        rescue Support::AllExceptionsExceptOnesWeMustNotRescue => e
          set_exception(e)
        ensure
          @example_group_instance = nil
        end

        finish(reporter)
      ensure
        execution_result.ensure_timing_set(clock)
        Rex.current_example = nil
      end

      if Rex::Support::Ruby.jruby? || RUBY_VERSION.to_f < 1.9
        AllExceptionsExcludingDangerousOnesOnRubiesThatAllowIt = Exception
      else
        AllExceptionsExcludingDangerousOnesOnRubiesThatAllowIt = Support::AllExceptionsExceptOnesWeMustNotRescue
      end

      class Procsy
        attr_reader :example

        Example.public_instance_methods(false).each do |name|
          name_sym = name.to_sym
          next if name_sym == :run || name_sym == :inspect || name_sym == :to_s

          define_method(name) { |*a, &b| @example.__send__(name, *a, &b) }
        end

        Proc.public_instance_methods(false).each do |name|
          name_sym = name.to_sym
          next if name_sym == :call || name_sym == :inspect || name_sym == :to_s || name_sym == :to_proc

          define_method(name) { |*a, &b| @proc.__send__(name, *a, &b) }
        end

        def call(*args, &block)
          @executed = true
          @proc.call(*args, &block)
        end
        alias run call

        def to_proc
          method(:call).to_proc
        end

        def initialize(example, &block)
          @example  = example
          @proc     = block
          @executed = false
        end

        def wrap(&block)
          self.class.new(example, &block)
        end

        def executed?
          @executed
        end

        def inspect
          @example.inspect.gsub('Example', 'Example::Procsy')
        end
      end

      def display_exception
        @exception || execution_result.pending_exception
      end

      def display_exception=(ex)
        if pending? && !(Pending::PendingExampleFixedError === ex)
          @exception = nil
          execution_result.pending_fixed = false
          execution_result.pending_exception = ex
        else
          @exception = ex
        end
      end


      def set_exception(exception)
        return self.display_exception = exception unless display_exception

        unless Rex::Core::MultipleExceptionError === display_exception
          self.display_exception = Rex::Core::MultipleExceptionError.new(display_exception)
        end

        display_exception.add exception
      end

      def set_aggregate_failures_exception(exception)
        return set_exception(exception) unless display_exception

        exception = Rex::Core::MultipleExceptionError::InterfaceTag.for(exception)
        exception.add display_exception
        self.display_exception = exception
      end


      def fail_with_exception(reporter, exception)
        start(reporter)
        set_exception(exception)
        finish(reporter)
      end

      def skip_with_exception(reporter, exception)
        start(reporter)
        Pending.mark_skipped! self, exception.argument
        finish(reporter)
      end

      def instance_exec(*args, &block)
        @example_group_instance.instance_exec(*args, &block)
      end

      # The example as it was before it ran, so it can run again.
      def reset_for_rerun
        @exception = nil
        @metadata[:execution_result] = ExecutionResult.new
      end

    private

      def hooks
        example_group_instance.singleton_class.hooks
      end

      def with_around_example_hooks
        Rex.current_scope = :before_example_hook
        hooks.run(:around, :example, self) { yield }
      rescue Support::AllExceptionsExceptOnesWeMustNotRescue => e
        set_exception(e)
      end

      def start(reporter)
        reporter.example_started(self)
        execution_result.started_at = clock.now
      end

      def finish(reporter)
        pending_message = execution_result.pending_message

        if @exception
          execution_result.exception = @exception
          record_finished :failed, reporter
          reporter.example_failed self
          false
        elsif pending_message
          execution_result.pending_message = pending_message
          record_finished :pending, reporter
          reporter.example_pending self
          true
        else
          record_finished :passed, reporter
          reporter.example_passed self
          true
        end
      end

      def record_finished(status, reporter)
        execution_result.record_finished(status, clock.now)
        reporter.example_finished(self)
      end

      def run_before_example
        @example_group_instance.setup_mocks_for_rspec
        hooks.run(:before, :example, self)
      end

      def with_around_and_singleton_context_hooks
        singleton_context_hooks_host = example_group_instance.singleton_class
        singleton_context_hooks_host.run_before_context_hooks(example_group_instance)
        with_around_example_hooks { yield }
      ensure
        singleton_context_hooks_host.run_after_context_hooks(example_group_instance)
      end

      def run_after_example
        assign_generated_description if defined?(::Rex::Matchers)
        hooks.run(:after, :example, self)
        verify_mocks
      ensure
        @example_group_instance.teardown_mocks_for_rspec
      end

      def verify_mocks
        @example_group_instance.verify_mocks_for_rspec if mocks_need_verification?
      rescue Support::AllExceptionsExceptOnesWeMustNotRescue => e
        set_exception(e)
      end

      def mocks_need_verification?
        exception.nil? || execution_result.pending_fixed?
      end

      def assign_generated_description
        if metadata[:description].empty? && (description = generate_description)
          metadata[:description] = description
          metadata[:full_description] += description
        end
      ensure
        Rex::Matchers.clear_generated_description
      end

      def generate_description
        Rex::Matchers.generated_description
      rescue Support::AllExceptionsExceptOnesWeMustNotRescue => e
        location_description + " (Got an error when generating description " \
          "from matcher: #{e.class}: #{e.message} -- #{e.backtrace.first})"
      end

      def location_description
        "example at #{location}"
      end

      class ExecutionResult
        include HashImitatable

        attr_accessor :status

        attr_accessor :exception

        attr_accessor :started_at

        attr_accessor :finished_at

        attr_accessor :run_time

        attr_accessor :pending_message

        attr_accessor :pending_exception

        attr_accessor :pending_fixed

        def pending_fixed?
          !!pending_fixed
        end

        def example_skipped?
          status == :pending && !pending_exception
        end

        def record_finished(status, finished_at)
          self.status = status
          calculate_run_time(finished_at)
        end

        def ensure_timing_set(clock)
          calculate_run_time(clock.now) unless finished_at
        end

      private

        def calculate_run_time(finished_at)
          self.finished_at = finished_at
          self.run_time    = (finished_at - started_at).to_f
        end

        def hash_for_delegation
          super.tap do |hash|
            hash[:status] &&= status.to_s
          end
        end

        def set_value(name, value)
          value &&= value.to_sym if name == :status
          super(name, value)
        end

        def get_value(name)
          if name == :status
            status.to_s if status
          else
            super
          end
        end

        def issue_deprecation(_method_name, *_args)
          Rex.deprecate("Treating `metadata[:execution_result]` as a hash",
                          :replacement => "the attributes methods to access the data")
        end
      end
    end

    class SuiteHookContext < Example
      def initialize(hook_description, reporter)
        super(AnonymousExampleGroup, hook_description, {})
        @example_group_instance = AnonymousExampleGroup.new
        @reporter = reporter
      end

      def set_exception(exception)
        reporter.notify_non_example_exception(exception, "An error occurred in #{description}.")
        Rex.world.wants_to_quit = true
      end
    end
  end
end
