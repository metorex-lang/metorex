# A Struct member written in a loop that runs often enough to be compiled.
Reading = Struct.new(:sensor, :level)

def record(reading, times)
  step = 0
  while step < times
    reading.level = step
    step += 1
  end
end

reading = Reading.new(:north, 0)
record(reading, 1000)
p reading.to_a
p reading.instance_variables
reading.freeze
begin
  record(reading, 1)
rescue FrozenError => error
  p error.class
end
