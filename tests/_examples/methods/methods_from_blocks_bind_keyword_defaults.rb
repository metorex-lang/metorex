original_warn = Kernel.method(:warn)
Kernel.define_singleton_method(:warn) { |_msg, **| :replaced }
p Kernel.warn("one")
Kernel.define_singleton_method(:warn, original_warn)
Kernel.warn("two")
Kernel.warn("three", uplevel: 0)

count = 0
racing = ->(_msg, **) { sleep(0.01) && (count += 1) }
Kernel.define_singleton_method(:warn, &racing)
p Kernel.warn("four", category: :deprecated)
lock = Mutex.new
Array.new(3) { Thread.new { lock.synchronize { Kernel.warn("five", category: :deprecated) } } }.each(&:join)
p count
Kernel.define_singleton_method(:warn, original_warn)
Kernel.warn("six", category: :deprecated)
p :done

captured = :missing
Kernel.define_singleton_method(:warn) { |_msg, category: nil, **| captured = category }
Kernel.warn("seven", category: :deprecated)
p captured
Kernel.define_singleton_method(:warn) { |first, second = :second, category: nil| [first, second, category] }
p Kernel.warn(1), Kernel.warn(1, 2, category: 3)
p Kernel.warn(1, 2)
Kernel.define_singleton_method(:warn) { |first = 0, *rest, last, label: "default", **others, &block| [first, rest, last, label, others] }
p Kernel.warn(9), Kernel.warn(1, 2, 3, other: 4)
