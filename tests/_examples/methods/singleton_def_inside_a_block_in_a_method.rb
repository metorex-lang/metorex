# A `def self.name` run inside a block within an instance method defines
# the method on that instance, and a bare call to it then finds it.
class Recorder
  def run
    yield
  end

  def check
    run do
      def self.helper(value)
        [:helper, value]
      end

      p helper(1)
      p respond_to?(:helper)
    end
  end
end

recorder = Recorder.new
recorder.check
p recorder.singleton_methods
p Recorder.new.respond_to?(:helper)
