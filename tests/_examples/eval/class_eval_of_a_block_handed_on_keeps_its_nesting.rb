def build(&block)
  Class.new.class_eval(&block)
end
module Holder
  def self.build2(&block)
    Class.new.class_eval(&block)
  end
end
class Outer
  def a; build { NOPE } ; rescue NameError => e; p e.message; end
  def b; Holder.build2 { NOPE } ; rescue NameError => e; p e.message; end
  def c; Class.new.class_eval { NOPE } ; rescue NameError => e; p e.message; end
  def d; [1].each { NOPE } ; rescue NameError => e; p e.message; end
end
o = Outer.new
o.a; o.b; o.c; o.d
def eval_it(&block) = Class.new.class_eval(&block)
def exec_it(&block) = Class.new.class_exec(&block)
def inst_it(&block) = Object.new.instance_eval(&block)
class Outer
  def nestings
    eval_it { p Module.nesting }
    exec_it { p Module.nesting }
    inst_it { p Module.nesting }
  end
end
Outer.new.nestings
