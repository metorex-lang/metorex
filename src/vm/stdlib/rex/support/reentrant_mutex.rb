# frozen_string_literal: true

module Rex
  module Support
    class ReentrantMutex
      def initialize
        @owner = nil
        @count = 0
        @mutex = Mutex.new
      end

      def synchronize
        enter
        yield
      ensure
        exit
      end

    private

      if RUBY_VERSION.to_f >= 3.0
        def enter
          @mutex.lock unless @mutex.owned?
          @count += 1
        end

        def exit
          unless @mutex.owned?
            raise ThreadError, "Attempt to unlock a mutex which is locked by another thread/fiber"
          end
          @count -= 1
          @mutex.unlock if @count == 0
        end
      else
        def enter
          @mutex.lock if @owner != Thread.current
          @owner = Thread.current
          @count += 1
        end

        def exit
          @count -= 1
          return unless @count == 0
          @owner = nil
          @mutex.unlock
        end
      end
    end

    if defined? ::Mutex
      class Mutex < ::Mutex
        NEW_MUTEX_METHOD = Mutex.method(:new)

        def self.new
          NEW_MUTEX_METHOD.call
        end
      end
    else # For 1.8.7
    end
  end
end
