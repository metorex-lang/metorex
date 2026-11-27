class Recorder
  def self.method_added(name)
    puts "method_added #{name}"
  end

  def singleton_method_added(name)
    puts "singleton_method_added #{name}"
  end
end

recorder = Recorder.new
singleton = class << recorder; self; end
singleton.send(:define_method, :run) { :stubbed }
singleton.send(:alias_method, :run_before, :run)
singleton.send(:undef_method, :run_before)
def recorder.finish = :done
class << recorder
  alias run_again run
end
p recorder.run
p recorder.finish
p recorder.run_again
