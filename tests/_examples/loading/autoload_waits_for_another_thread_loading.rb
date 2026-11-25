# A thread that reaches a constant another thread is still autoloading waits
# for that load to finish, so it never sees the class half defined, as
# websocket-driver's Hybi is reached by several server threads at once.
require "tmpdir"

DIRECTORY = Dir.mktmpdir
File.write(File.join(DIRECTORY, "framed_driver.rb"), <<~RUBY)
  class Driver
    class Framed < Driver
      LOADING.push(:started)
      sleep 0.2

      def initialize
        super
        @stage = :framed
      end
    end
  end
RUBY

LOADING = Queue.new

class Driver
  autoload :Framed, File.join(DIRECTORY, "framed_driver")

  def initialize
    @stage = :plain
  end

  attr_reader :stage

  def self.build = Framed.new
end

first = Thread.new { Driver::Framed.new.stage }
LOADING.pop
second = Thread.new { Driver::Framed.new.stage }
third = Thread.new { Driver.build.stage }
p [first.value, second.value, third.value]
p Driver.autoload?(:Framed)

FileUtils.rm_rf(DIRECTORY)
