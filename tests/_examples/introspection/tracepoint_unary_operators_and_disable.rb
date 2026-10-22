# MRI writes enable and disable in Ruby, and a trace hears only the call of
# disable, from where MRI defines it.
heard = []
number = 3
ratio = 1.5
flag = true
trace = TracePoint.new(:call, :return, :c_call, :c_return) do |tp|
  heard << [tp.event, tp.method_id, File.basename(tp.path)]
end
trace.enable
negated = !flag
opposite = -number
same = +number
turned = -ratio
trace.disable
heard.each { |event| p event }
p negated, opposite, same, turned
