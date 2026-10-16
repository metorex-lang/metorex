# An object only a C extension's struct holds stays alive through a
# collection, and is freed by the first one after the struct lets it go.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require(build_extension("c_holding.c", "c_holding", directory))

# A finalizer made here holds no object it is defined for.
def finalizer
  proc { puts "held object freed" }
end

def hold_new_object
  held = Object.new
  held.instance_variable_set(:@label, "held by C")
  ObjectSpace.define_finalizer(held, finalizer)
  CHolding.hold(held)
end

holder = hold_new_object
GC.start
puts("after the first collection")
p(CHolding.held(holder).instance_variable_get(:@label))
CHolding.let_go(holder)
GC.start
puts("after the second collection")
p(CHolding.held(holder))

FileUtils.rm_rf(directory)
