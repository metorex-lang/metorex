# A `method_added` a module defines on itself hears of the methods defined
# in a `class_exec` block and in its own body alike.
module Host
  def self.method_added(name)
    puts "added #{name}"
  end
end

Host.class_exec do
  def receive = :receive
end

module Host
  def allow = :allow
end

Host.module_eval { def stub = :stub }
