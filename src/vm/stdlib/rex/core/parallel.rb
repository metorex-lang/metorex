module Rex
  module Core
    # `--jobs N`: the top-level groups run in N forked workers. Each worker
    # records what happened in each group, its output included, and the
    # parent replays the groups to the formatters in the order a serial run
    # would have run them, so the output reads the same.
    module Parallel
      # The coverage counts each worker sent back.
      def self.worker_coverage
        @worker_coverage ||= []
      end

      # Sends what the run reports to `recorder` in place of the reporter
      # the configuration built.
      def self.report_to(recorder)
        Rex.configuration.instance_variable_set(:@reporter, recorder)
      end

      # An exception that cannot be sent between processes as it is, sent
      # as its class name, message, backtrace, and cause.
      class RemoteError < StandardError
        attr_reader :remote_class_name

        def self.from(error)
          remote = new(error.message.to_s)
          remote.instance_variable_set(:@remote_class_name, error.class.name || "(anonymous error class)")
          remote.set_backtrace(error.backtrace || [])
          remote
        end
      end

      def self.portable(error)
        return nil if error.nil?

        Marshal.load(Marshal.dump(error))
      rescue TypeError, ArgumentError, NoMethodError
        RemoteError.from(error)
      end

      # What a worker writes, one Marshal packet after its size, to the
      # pipe the parent reads.
      class Pipe
        def initialize
          @read_io, @write_io = IO.pipe
          @read_io.binmode
          @write_io.binmode
        end

        attr_reader :read_io

        def close_reading
          @read_io.close
        end

        def close_writing
          @write_io.close
        end

        def send_message(message)
          packet = Marshal.dump(message)
          @write_io.write("#{packet.bytesize}\n#{packet}")
          @write_io.flush
        end

        # The next message, or nil once the worker has closed its end.
        def receive
          size = @read_io.gets
          return nil unless size

          Marshal.load(@read_io.read(Integer(size)))
        end
      end

      # A stream that records what is written to it as an event, so text an
      # example prints keeps its place among the example's events.
      class RecordingStream
        def initialize(recorder, kind)
          @recorder = recorder
          @kind = kind
        end

        def write(*texts)
          text = texts.join
          @recorder.record(@kind, text) unless text.empty?
          text.bytesize
        end

        def <<(text)
          write(text.to_s)
          self
        end

        def print(*texts)
          write(*texts.map(&:to_s))
          nil
        end

        def puts(*texts)
          if texts.empty?
            write("\n")
          else
            texts.flatten.each { |text| write(text.to_s.end_with?("\n") ? text.to_s : "#{text}\n") }
          end
          nil
        end

        def printf(format_string, *arguments)
          write(format(format_string, *arguments))
          nil
        end

        def flush
          self
        end

        def sync
          true
        end

        def sync=(_value); end

        def tty?
          false
        end
        alias isatty tty?

        def fileno
          nil
        end
      end

      # Stands in for the reporter in a worker, recording each event.
      class Recorder
        attr_reader :events

        def initialize
          @events = []
          @group_starts = {}
        end

        def record(*event)
          @events << event
        end

        def take_events
          taken = @events
          @events = []
          taken
        end

        def set_up?
          true
        end

        def example_group_started(group)
          @group_starts[group] = Rex::Core::Time.now
          record(:group_started, group.id)
        end

        def example_group_finished(group)
          started = @group_starts.delete(group)
          record(:group_finished, group.id, started ? (Rex::Core::Time.now - started).to_f : 0)
        end

        def example_started(example)
          record(:example_started, example.id)
        end

        def example_finished(example)
          result = example.execution_result
          record(:example_finished, example.id,
                 status: result.status,
                 exception: Parallel.portable(result.exception),
                 pending_message: result.pending_message,
                 pending_exception: Parallel.portable(result.pending_exception),
                 pending_fixed: result.pending_fixed,
                 run_time: result.run_time,
                 example_exception: Parallel.portable(example.exception),
                 metadata: example.metadata.slice(:description, :full_description, :pending, :skip))
        end

        def example_passed(example)
          record(:example_passed, example.id)
        end

        def example_failed(example)
          record(:example_failed, example.id)
        end

        def example_pending(example)
          record(:example_pending, example.id)
        end

        def message(text)
          record(:message, text)
        end

        def deprecation(hash)
          record(:deprecation, hash)
        end

        def notify_non_example_exception(exception, description)
          Rex.world.non_example_failure = true
          record(:non_example_exception, Parallel.portable(exception), description)
        end

        # A worker sees only its own failures, so the parent decides when
        # `--fail-fast` stops the run.
        def fail_fast_limit_met?
          false
        end
      end

      # Runs `groups` in `jobs` workers and replays them through `reporter`
      # in order. Answers whether the run was cut short.
      def self.run(groups, jobs, reporter)
        world = Rex.world
        groups_by_id = {}
        examples_by_id = {}
        world.example_groups.flat_map(&:descendants).each do |group|
          groups_by_id[group.id] = group
          group.examples.each { |example| examples_by_id[example.id] = example }
        end

        workers = Array.new([jobs, groups.size].min) do |worker_index|
          pipe = Pipe.new
          pid = fork { run_worker(groups, worker_index, jobs, pipe) }
          pipe.close_writing
          { pid: pid, pipe: pipe, done: false }
        end

        replay = Replay.new(reporter, groups_by_id, examples_by_id)
        recorded = {}
        next_index = 0
        open_workers = workers.dup
        stopped = false
        until open_workers.empty? || stopped
          ready, = IO.select(open_workers.map { |worker| worker[:pipe].read_io })
          ready.each do |read_io|
            worker = open_workers.find { |each_worker| each_worker[:pipe].read_io.equal?(read_io) }
            message = worker[:pipe].receive
            case message
            in [:group, index, events, quit]
              recorded[index] = [events, quit]
            in [:done, quit, coverage]
              world.wants_to_quit ||= quit
              worker_coverage << coverage if coverage
              open_workers.delete(worker)
            in nil
              open_workers.delete(worker)
            end
          end
          # A group a worker was interrupted in is the last one shown, as it
          # is in a serial run.
          while recorded.key?(next_index) && !stopped
            events, quit = recorded.delete(next_index)
            stopped = replay.call(events) || quit
            next_index += 1
          end
        end

        if stopped
          world.wants_to_quit = true
          workers.each { |worker| Process.kill("TERM", worker[:pid]) rescue nil }
        end
        workers.each do |worker|
          Process.wait(worker[:pid]) rescue nil
          worker[:pipe].close_reading
        end
        stopped
      end

      # Runs the whole suite again in one forked worker, in the same order,
      # and warns about each example whose outcome differs from the parallel
      # run's, which points at state it shares with a group another worker
      # ran.
      def self.verify(groups, world)
        parallel_statuses = world.all_examples.to_h { |example| [example.id, example.execution_result.status] }
        pipe = Pipe.new
        pid = fork do
          recorder = Recorder.new
          report_to(recorder)
          world.wants_to_quit = false
          $stdout = RecordingStream.new(recorder, :output)
          $stderr = RecordingStream.new(recorder, :error_output)
          world.all_examples.each(&:reset_for_rerun)
          groups.each { |group| group.run(recorder) }
          pipe.send_message(world.all_examples.to_h { |example| [example.id, example.execution_result.status] })
        ensure
          exit!(0)
        end
        pipe.close_writing
        serial_statuses = pipe.receive || {}
        Process.wait(pid)
        pipe.close_reading

        differing = parallel_statuses.reject { |id, status| serial_statuses[id] == status || status.nil? }
        return if differing.empty?

        lines = ["WARNING: These examples had a different outcome when the suite ran serially in the same order, " \
                     "which points at state they share with a group another worker ran:"]
        differing.each do |id, status|
          lines << "  #{id} #{status} in parallel and #{serial_statuses[id] || 'did not run'} serially"
        end
        Rex.configuration.error_stream.puts(lines.join("\n"))
      end

      def self.run_worker(groups, worker_index, jobs, pipe)
        # A worker reports only what it ran, so the counts it was forked
        # with, which the parent reports, are cleared.
        ::Coverage.result(stop: false, clear: true) if Rex.configuration.coverage
        recorder = Recorder.new
        report_to(recorder)
        $stdout = RecordingStream.new(recorder, :output)
        $stderr = RecordingStream.new(recorder, :error_output)
        groups.each_with_index do |group, index|
          next unless index % jobs == worker_index

          group.run(recorder)
          pipe.send_message([:group, index, recorder.take_events, Rex.world.wants_to_quit ? true : false])
          break if Rex.world.wants_to_quit
        end
        coverage = Rex.configuration.coverage ? CoverageReport.collect : nil
        pipe.send_message([:done, Rex.world.wants_to_quit ? true : false, coverage])
      ensure
        exit!(0)
      end

      # Hands one group's recorded events to the reporter, as the group's
      # run would have.
      class Replay
        def initialize(reporter, groups_by_id, examples_by_id)
          @reporter = reporter
          @groups_by_id = groups_by_id
          @examples_by_id = examples_by_id
        end

        # Answers true once `--fail-fast` says the run stops here.
        def call(events)
          events.each do |event|
            case event
            in [:group_started, id]
              @reporter.example_group_started(@groups_by_id.fetch(id))
            in [:group_finished, id, duration]
              group = @groups_by_id.fetch(id)
              profiler = @reporter.instance_variable_get(:@profiler)
              profiler.durations[group] = duration if profiler
              @reporter.example_group_finished(group)
            in [:example_started, id]
              @reporter.example_started(@examples_by_id.fetch(id))
            in [:example_finished, id, data]
              example = @examples_by_id.fetch(id)
              apply(example, data)
              @reporter.example_finished(example)
            in [:example_passed, id]
              @reporter.example_passed(@examples_by_id.fetch(id))
            in [:example_pending, id]
              @reporter.example_pending(@examples_by_id.fetch(id))
            in [:example_failed, id]
              @reporter.example_failed(@examples_by_id.fetch(id))
              return true if @reporter.fail_fast_limit_met?
            in [:output, text]
              $stdout.write(text)
            in [:error_output, text]
              $stderr.write(text)
            in [:message, text]
              @reporter.message(text)
            in [:deprecation, hash]
              @reporter.deprecation(hash)
            in [:non_example_exception, exception, description]
              @reporter.notify_non_example_exception(exception, description)
            end
          end
          false
        end

        private

        def apply(example, data)
          result = example.execution_result
          result.status = data[:status]
          result.exception = data[:exception]
          result.pending_message = data[:pending_message]
          result.pending_exception = data[:pending_exception]
          result.pending_fixed = data[:pending_fixed]
          result.run_time = data[:run_time]
          example.instance_variable_set(:@exception, data[:example_exception])
          example.metadata.update(data[:metadata])
        end
      end
    end
  end
end
