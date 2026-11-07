def run_with(flag)
  [1].each do
    begin
      if flag
        failed = true
      else
        :fine
      end
    ensure
      yield(failed)
      [2].each { yield(failed ? :failed : :passed) }
    end
  end
end

run_with(true) { |seen| p seen }
run_with(false) { |seen| p seen }

def nested
  begin
    if false
      noted = 1
    end
  ensure
    tap { return [noted, defined?(noted)] }
  end
end
p nested
