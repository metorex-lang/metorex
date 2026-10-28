# A method defined from a block shares the variables the block closes over
# with every Method object taken from it. A collection that frees such a
# Method object leaves those variables to the method.

class Shared
  def self.define_reader(name)
    define_method(:reader) { name }
  end

  def self.drop_a_copy
    copy = [instance_method(:reader)]
    copy << copy
    nil
  end
end

Shared.define_reader("kept name")
Shared.drop_a_copy
GC.start
puts Shared.new.reader.inspect
