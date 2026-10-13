module Rex
  module Core
    module SharedContext
      def included(group)
        __shared_context_recordings.each do |recording|
          recording.playback_onto(group)
        end
      end

      def __shared_context_recordings
        @__shared_context_recordings ||= []
      end

      Recording = Struct.new(:method_name, :args, :block) do
        def playback_onto(group)
          group.__send__(method_name, *args, &block)
        end
      end

      def self.record(methods)
        methods.each do |meth|
          define_method(meth) do |*args, &block|
            __shared_context_recordings << Recording.new(meth, args, block)
          end
        end
      end

      record [:describe, :context] + Hooks.instance_methods(false) +
        MemoizedHelpers::ClassMethods.instance_methods(false)
    end
  end
  SharedContext = Core::SharedContext
end
