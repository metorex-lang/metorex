class Nest; end
Object.new.instance_exec do
  class Nest
    TARGET = Object.new
    def TARGET.defs_method = :called
    p [:class_singleton, singleton_class.instance_methods(false)]
    p [:class_own, instance_methods(false)]
  end
  p [:receiver_singleton, singleton_methods]
end
proc do
  class Nest
    OTHER = Object.new
    def OTHER.other_method = :called
  end
end.call
p Nest::OTHER.singleton_methods
