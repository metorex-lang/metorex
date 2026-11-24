procs = [proc { }, proc { }]
p procs[0] == procs[1]
p procs[0].eql?(procs[1])
p procs[0].hash == procs[1].hash
p (procs & procs).size
p ([procs[0]] & procs).size
original = proc { |value| value }
def handed(&block)
  block
end
p original == original.dup
p original == original.clone
p handed(&original) == original
made = 2.times.map { proc { } }
p made[0] == made[1]
p lambda { } == lambda { }
