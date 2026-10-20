# A subclass reaches a class method its parent's singleton class redefines,
# and the original again once the redefinition is removed and the stashed
# copy is aliased back.
klass = Class.new do
  def self.say_hello
    @hello if defined?(@hello)
  end
end
subclass = Class.new(klass)
subclass.instance_variable_set(:@hello, "Hello")
p subclass.say_hello

klass.singleton_class.class_exec do
  alias_method :__stashed_say_hello, :say_hello
  define_method(:say_hello) { "Howdy" }
end
p subclass.say_hello

klass.singleton_class.class_exec do
  remove_method :say_hello
  alias_method :say_hello, :__stashed_say_hello
  remove_method :__stashed_say_hello
end
p subclass.say_hello
p klass.say_hello
