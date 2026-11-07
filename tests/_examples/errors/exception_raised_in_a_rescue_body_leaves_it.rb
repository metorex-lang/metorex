def handled
  attempts = 0
  begin
    raise "first"
  rescue => problem
    attempts += 1
    raise "from the rescue body" if attempts < 5
  end
  attempts
end

begin
  p handled
rescue => outer
  p outer.message
end

def handled_with_ensure
  begin
    raise "first"
  rescue => problem
    raise TypeError, "second"
  ensure
    puts "ensured"
  end
rescue TypeError => outer
  outer.message
end

p handled_with_ensure

class Holder
  def run
    yield
  rescue => problem
    record(problem)
  end

  def record(problem)
    @failures << problem
  end
end

begin
  Holder.new.run { raise "inside" }
rescue NoMethodError => outer
  p outer.class
end
