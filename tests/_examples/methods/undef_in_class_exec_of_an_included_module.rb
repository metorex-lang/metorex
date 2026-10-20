# Methods a module defines through `class_exec` show in `method_defined?`,
# and `undef` written in another `class_exec` removes them from every
# object of a class that includes the module.
module ExampleMethods; end

class Group
  include ExampleMethods
end

ExampleMethods.class_exec do
  def allow(target) = "allow #{target}"
  def receive(name) = "receive #{name}"
end

group = Group.new
p ExampleMethods.method_defined?(:allow)
p group.respond_to?(:receive)
puts group.allow(:ledger)

ExampleMethods.class_exec do
  undef allow
  undef receive
end

p ExampleMethods.method_defined?(:allow)
p group.respond_to?(:receive)
begin
  group.allow(:ledger)
rescue NoMethodError => error
  puts error.message
end
