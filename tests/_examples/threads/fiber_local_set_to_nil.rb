thread = Thread.new {}.join
thread[:value] = 1
p thread.key?(:value)
p(thread[:value] = nil)
p thread.key?(:value)
p thread.keys
Thread.current[:other] = 2
Thread.current[:other] = nil
p Thread.current.keys.include?(:other)
