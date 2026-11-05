# A module prepended to Process's singleton class sees every fork through
# _fork, and its super starts the child.
module ForkWatcher
  def _fork
    pid = super
    $stdout.puts("child saw the hook") if pid == 0
    pid
  end
end
Process.singleton_class.prepend(ForkWatcher)

$stdout.sync = true
pid = fork { exit!(0) }
Process.wait(pid)
pid = Process.fork { exit!(0) }
Process.wait(pid)
puts "parent done"
