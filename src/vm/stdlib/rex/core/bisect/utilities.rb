module Rex
  module Core
    module Bisect
      ExampleSetDescriptor = Struct.new(:all_example_ids, :failed_example_ids)

      class BisectFailedError < StandardError
        def self.for_failed_spec_run(spec_output)
          new("Failed to get results from the spec run. Spec run output:\n\n" +
              spec_output)
        end
      end

      class Notifier
        def initialize(formatter)
          @formatter = formatter
        end

        def publish(event, *args)
          return unless @formatter.respond_to?(event)
          notification = Notifications::CustomNotification.for(*args)
          @formatter.__send__(event, notification)
        end
      end

      class Channel
        if String.method_defined?(:encoding)
          MARSHAL_DUMP_ENCODING = Marshal.dump("").encoding
        end

        def initialize
          @read_io, @write_io = IO.pipe

          if defined?(MARSHAL_DUMP_ENCODING) && IO.method_defined?(:set_encoding)
            @write_io.set_encoding MARSHAL_DUMP_ENCODING
          end
        end

        def send(message)
          packet = Marshal.dump(message)
          @write_io.write("#{packet.bytesize}\n#{packet}")
        end

        def receive
          packet_size = Integer(@read_io.gets)
          Marshal.load(@read_io.read(packet_size))
        end

        def close
          @read_io.close
          @write_io.close
        end
      end
    end
  end
end
