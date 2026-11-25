# A C extension giving a Hash a default Proc by setting its default and its
# PROC_DEFAULT flag, telling a lambda from a proc, and comparing pairs of
# objects through rb_exec_recursive_paired.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require(build_extension("c_proc_defaults.c", "c_proc_defaults", directory))
defaults = CProcDefaults.new

p(defaults.user_flags)

counted = {}
defaults.set_default(counted, proc { |hash, key| hash[key] = key.to_s * 2 }, true)
p(counted[:ab])
p(counted)
p(counted.default_proc.class)
p(defaults.proc_default?(counted))

plain = {}
defaults.set_default(plain, 5, false)
p([plain[:missing], plain.default_proc])
p(defaults.proc_default?(plain))

held = Hash.new { |_hash, key| key }
p(defaults.proc_default?(held))
defaults.set_default(held, held.default_proc, false)
p(held.default_proc)
p(held[:missing].class)

p([defaults.lambda?(lambda { 1 }), defaults.lambda?(proc { 1 }), defaults.lambda?(method(:puts).to_proc)])

p(defaults.recurse_paired(:left, [:right, :other]))
p(defaults.recurse_paired(:left, [:right, :right]))
p(defaults.recurse_paired(:left, []))

FileUtils.rm_rf(directory)
