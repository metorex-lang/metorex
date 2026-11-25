filter = proc { |c, a| p [c.class, a.class] }
o = Object.new
o.instance_exec(o, proc { 1 }, &filter)
o.instance_exec(1, proc { 1 }, &filter)
o.instance_exec(proc { 1 }, 2, &filter)
o.instance_exec(1, -> { 1 }, &filter)
filter.call(1, proc { 1 })
o.instance_exec(1, proc { 1 }) { |c, a| p [c.class, a.class] }
