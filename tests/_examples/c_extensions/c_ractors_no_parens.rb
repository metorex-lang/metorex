# A C extension keeping a value and a pointer apart for each Ractor under a
# key, and asking which Ractor runs and what Ruby may share between them.
require "tmpdir"
require_relative "build_helper"

Warning[:experimental] = false
directory = Dir.mktmpdir
require build_extension("c_ractors.c", "c_ractors", directory)
ractors = CRactors.new

p ractors.stored
ractors.store("main")
p ractors.stored
p Ractor.new(ractors) { |held| [held.stored, held.store(:other), held.stored] }.value
p ractors.stored
p ractors.shareable([+"text"])
p Ractor.new(ractors) { |held| held.shareable(1) }.value
