module Rex::Core
  class Reporter
    RSPEC_NOTIFICATIONS = Set.new(
      [
        :close, :deprecation, :deprecation_summary, :dump_failures, :dump_pending,
        :dump_profile, :dump_summary, :example_failed, :example_group_finished,
        :example_group_started, :example_passed, :example_pending, :example_started,
        :message, :seed, :start, :start_dump, :stop, :example_finished
      ])

    def initialize(configuration)
      @configuration = configuration
      @listeners = Hash.new { |h, k| h[k] = Set.new }
      @examples = []
      @failed_examples = []
      @pending_examples = []
      @duration = @start = @load_time = nil
      @non_example_exception_count = 0
      @setup_default = lambda {}
      @setup = false
      @profiler = nil
    end

    attr_reader :examples, :failed_examples, :pending_examples

    def register_listener(listener, *notifications)
      notifications.each do |notification|
        @listeners[notification.to_sym] << listener
      end
      true
    end

    def prepare_default(loader, output_stream, deprecation_stream)
      @setup_default = lambda do
        loader.setup_default output_stream, deprecation_stream
      end
    end

    def registered_listeners(notification)
      @listeners[notification].to_a
    end

    def report(expected_example_count)
      start(expected_example_count)
      begin
        yield self
      ensure
        finish
      end
    end

    def exit_early(exit_code)
      report(0) { exit_code }
    end

    def start(expected_example_count, time=Rex::Core::Time.now)
      @start = time
      @load_time = (@start - @configuration.start_time).to_f
      notify :seed, Notifications::SeedNotification.new(@configuration.seed, seed_used?)
      notify :start, Notifications::StartNotification.new(expected_example_count, @load_time)
    end

    def message(message)
      notify :message, Notifications::MessageNotification.new(message)
    end

    def publish(event, options={})
      if RSPEC_NOTIFICATIONS.include? event
        raise "RSpec::Core::Reporter#publish is intended for sending custom " \
              "events not internal RSpec ones, please rename your custom event."
      end
      notify event, Notifications::CustomNotification.for(options)
    end

    def example_group_started(group)
      notify :example_group_started, Notifications::GroupNotification.new(group) unless group.descendant_filtered_examples.empty?
    end

    def example_group_finished(group)
      notify :example_group_finished, Notifications::GroupNotification.new(group) unless group.descendant_filtered_examples.empty?
    end

    def example_started(example)
      @examples << example
      notify :example_started, Notifications::ExampleNotification.for(example)
    end

    def example_finished(example)
      notify :example_finished, Notifications::ExampleNotification.for(example)
    end

    def example_passed(example)
      notify :example_passed, Notifications::ExampleNotification.for(example)
    end

    def example_failed(example)
      @failed_examples << example
      notify :example_failed, Notifications::ExampleNotification.for(example)
    end

    def example_pending(example)
      @pending_examples << example
      notify :example_pending, Notifications::ExampleNotification.for(example)
    end

    def deprecation(hash)
      notify :deprecation, Notifications::DeprecationNotification.from_hash(hash)
    end

    def notify_non_example_exception(exception, context_description)
      @configuration.world.non_example_failure = true
      @non_example_exception_count += 1

      example = Example.new(AnonymousExampleGroup, context_description, {})
      presenter = Formatters::ExceptionPresenter.new(exception, example, :indentation => 0)
      message presenter.fully_formatted(nil)
    end

    def finish
      close_after do
        examples_notification = Notifications::ExamplesNotification.new(self)
        stop(examples_notification)
        notify :start_dump,    Notifications::NullNotification
        notify :dump_pending,  examples_notification
        notify :dump_failures, examples_notification
        notify :deprecation_summary, Notifications::NullNotification
        unless mute_profile_output?
          notify :dump_profile, Notifications::ProfileNotification.new(@duration, @examples,
                                                                       @configuration.profile_examples,
                                                                       @profiler.example_groups)
        end
        notify :dump_summary, Notifications::SummaryNotification.new(@duration, @examples, @failed_examples,
                                                                     @pending_examples, @load_time,
                                                                     @non_example_exception_count)
        notify :seed, Notifications::SeedNotification.new(@configuration.seed, seed_used?)
      end
    end

    def close_after
      yield
    ensure
      close
    end

    def stop(notification)
      @duration = (Rex::Core::Time.now - @start).to_f if @start
      notify :stop, notification
    end

    def notify(event, notification)
      ensure_listeners_ready
      registered_listeners(event).each do |formatter|
        formatter.__send__(event, notification)
      end
    end

    def abort_with(msg, exit_status)
      message(msg)
      close
      exit!(exit_status)
    end

    def fail_fast_limit_met?
      return false unless (fail_fast = @configuration.fail_fast)

      if fail_fast == true
        @failed_examples.any?
      else
        fail_fast <= @failed_examples.size
      end
    end

  private

    def ensure_listeners_ready
      return if @setup

      @setup_default.call
      @profiler = Profiler.new
      register_listener @profiler, *Profiler::NOTIFICATIONS
      @setup = true
    end

    def close
      notify :close, Notifications::NullNotification
    end

    def mute_profile_output?
      !@configuration.profile_examples? || fail_fast_limit_met?
    end

    def seed_used?
      @configuration.seed && @configuration.seed_used?
    end
  end

  class NullReporter
    def self.method_missing(*)
    end
    private_class_method :method_missing
  end
end
