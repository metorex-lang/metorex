Hook = Struct.new(:block, :options)
body = proc { :ran }
first = Hook.new(body, { scope: :context })
second = Hook.new(body, { scope: :context })
p first == second, first.eql?(second), first.hash == second.hash
p ([first] - [second]).size
p [first, second].uniq.size
p({ first => 1 }.key?(second))
