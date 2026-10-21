trap("INT") { exit(1) }

begin
  Process.kill("INT", Process.pid)
  sleep 5
rescue SystemExit => error
  p error.status
  p error.success?
end

trap("INT") { exit }
begin
  Process.kill("INT", Process.pid)
  sleep 5
rescue SystemExit => error
  p error.status
end

class Coordinator
  def abort_on_interrupt
    trap("INT") do
      @aborted = true
      exit(1)
    end
  end

  def run
    abort_on_interrupt
    Process.kill("INT", Process.pid)
    sleep 5
  end
end

begin
  Coordinator.new.run
rescue SystemExit => error
  p error.status
end
