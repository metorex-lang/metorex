to_s = Object.instance_method(:to_s)
p to_s.bind_call(Object.new.singleton_class).match?(/\A#<Class:0x\h{16}>\z/)
p to_s.bind_call(Module.new).match?(/\A#<Module:0x\h{16}>\z/)
p to_s.bind_call(Class.new).match?(/\A#<Class:0x\h{16}>\z/)
