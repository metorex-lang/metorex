# A thread that requires a file another thread is still loading waits for
# that load to finish, so it sees everything the file defines, as Bundler's
# download threads each require psych. The waiting holds while the loading
# thread is inside a file the first one requires in turn, and for a library
# the interpreter carries, as psych is.
require "tmpdir"

Dir.mktmpdir do |directory|
  File.write(File.join(directory, "slow_library.rb"), <<~RUBY)
    module SlowLibrary
      sleep 0.2
      def self.ready
        :ready
      end
    end
  RUBY
  File.write(File.join(directory, "slow_part.rb"), <<~RUBY)
    sleep 0.2
  RUBY
  File.write(File.join(directory, "nesting_library.rb"), <<~RUBY)
    module NestingLibrary
    end
    require "slow_part"
    module NestingLibrary
      def self.ready
        :ready
      end
    end
  RUBY
  $LOAD_PATH.unshift(directory)

  %w[slow_library nesting_library].each do |name|
    results = Array.new(3) do
      Thread.new do
        loaded = require name
        [loaded, Object.const_get(name.split("_").map(&:capitalize).join).respond_to?(:ready)]
      end
    end.map(&:value)
    p results.map(&:last)
    p results.count(&:first)
  end

  # psych requires date before it defines its methods, and this date pauses
  # the thread loading psych there.
  File.write(File.join(directory, "date.rb"), <<~RUBY)
    sleep 0.2
    $LOADED_FEATURES.delete(__FILE__)
    $LOAD_PATH.delete(File.dirname(__FILE__))
    require "date"
  RUBY
  results = Array.new(3) do
    Thread.new do
      require "psych"
      Psych.respond_to?(:safe_load)
    end
  end.map(&:value)
  p results
end
