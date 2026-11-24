class Wrapped < String
end
safe = Wrapped.new("<br/>")
p "foo %{x} bar" % { x: safe }
p "foo %s bar" % [safe]
p format("%-8s|", safe)

def capture(&block)
  "abc123".sub(/([a-z]+)([0-9]+)/) do |*params|
    block.binding.eval("proc { |m| $~ = m }").call($~)
    block.call(*params)
  end
end
p capture { $2 + $1 }

procs = [proc { }, proc { }]
p procs[0] == procs[1]
p procs[0].eql?(procs[1])
p procs[0].hash == procs[1].hash
p (procs & procs).size
p ([procs[0]] & procs).size
