# A deprecated constant warns when read through a subclass, by its bare
# name, and at the top level, naming the class that holds it.
require "stringio"

Warning[:deprecated] = true
$stderr = StringIO.new

class Base
  OLD = 1
  deprecate_constant :OLD
end

class Child < Base
  def self.read_bare = OLD
end

TOP = 2
Object.deprecate_constant :TOP

reads = [Base::OLD, Child::OLD, Child.read_bare, Child.read_bare, TOP]
holder = Child
2.times { reads << holder::OLD }

warnings = $stderr.string
$stderr = STDERR
p reads
puts warnings.gsub(/^.*:(\d+): /, "line \\1: ")
