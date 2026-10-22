# Code run through a binding, and the blocks it makes, read the class
# variables of the class the binding was taken in.
class Holder
  @@count = 2
  def scope = binding
end

class Reader
  @@count = 9
  def through_a_block(scope) = scope.eval("Proc.new { @@count }").call
  def directly(scope) = scope.eval("@@count")
end

reader = Reader.new
p reader.through_a_block(Holder.new.scope), reader.directly(Holder.new.scope)
