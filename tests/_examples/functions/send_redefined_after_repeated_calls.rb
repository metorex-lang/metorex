class Tables
  class << self
    attr_accessor :keys
    private :keys
  end
  self.keys = [1, 2, 3]

  def read(times)
    klass = self.class
    Array.new(times) { klass.send :keys }.last
  end

  def size_of_name
    send :name_length
  end

  private

  def name_length
    7
  end
end

class SubTables < Tables
  self.keys = [4]
end

tables = Tables.new
p tables.read(3)
p SubTables.new.read(3)

class << Tables
  def send(name, *rest)
    [:intercepted, name]
  end
end

p tables.read(1)
p SubTables.new.read(1)

def tables.send(name, *rest)
  :own_send
end

reading = Tables.new
p Array.new(3) { reading.size_of_name }
p tables.size_of_name

class Tables
  def send(name, *rest)
    :class_send
  end
end

p reading.size_of_name

class Namespace < Module
end

namespace = Namespace.new
namespace.const_set(:LIMIT, 80)
p namespace.const_get(:LIMIT)
p namespace.is_a?(Module)
p Tables.new.is_a?(Module)
