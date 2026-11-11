class Outer; end
class Outer::File; end
blk = proc { File.name }
puts Outer::File.new.instance_exec(&blk)
Outer.class_exec { define_method(:fname) { File.name } }
class Inner < Outer
  File = 1
end
puts Inner.new.fname
m = Module.new
m.module_eval { def self.x; File.name; end }
puts m.x
