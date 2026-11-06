# A C extension adding event hooks, which hand it either the trace argument
# or the event's details, and removing them, then the array, string and
# integer helpers msgpack and puma call. It is built with hidden visibility,
# which leaves its Init function exported.
require "tmpdir"
require_relative "build_helper"

class Greeter
  def greet
    :hello
  end
end

directory = Dir.mktmpdir
require(build_extension("c_event_hooks.c", "c_event_hooks", directory, ["-fvisibility=hidden"]))
hooks = CEventHooks.new
greeter = Greeter.new

raw = []
hooks.add_raw(raw)
greeter.greet
p(hooks.remove_raw(raw))
greeter.greet
p(raw)
p(hooks.remove_raw(raw))

details = []
hooks.add_details(details)
greeter.greet
greeter.hash
p(hooks.remove_details)
greeter.greet
p(details.map { |event, receiver, method_id, klass| [event, receiver.class, method_id, klass] })

p(hooks.resize([1, 2, 3], 5))
p(hooks.resize([1, 2, 3], 1))
report { hooks.resize([1].freeze, 2) }
text = +"before"
p(hooks.replace(text, "after").equal?(text))
p(text)
p(hooks.unsigned_long_long(2**64 - 1))
p(hooks.unsigned_long_long(-(2**63)))
report { hooks.unsigned_long_long(-(2**63) - 1) }
report { hooks.unsigned_long_long(2**64) }
p(hooks.fixnum_as_unsigned(42))
p(hooks.frozen_raw("thawed".dup))
p(hooks.frozen_raw("frozen".freeze))
p(hooks.largest_fixnum == 2**62 - 1)
report { hooks.end_of_file }

FileUtils.rm_rf(directory)
