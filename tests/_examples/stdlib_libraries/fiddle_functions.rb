# Fiddle::Function calls a C function at an address with the argument and
# return types it was declared with.
require "fiddle"

libc = Fiddle::Handle::DEFAULT
strlen = Fiddle::Function.new(libc["strlen"], [Fiddle::TYPE_VOIDP], Fiddle::TYPE_SIZE_T)
p(strlen.call("hello"))
p([strlen.ptr == libc["strlen"], strlen.to_i == strlen.ptr, strlen.abi, strlen.name, strlen.need_gvl?])

begin
  strlen.call
rescue ArgumentError => error
  p(error.message)
end

absolute = Fiddle::Function.new(libc["abs"], [Fiddle::TYPE_INT], Fiddle::TYPE_INT)
p(absolute.call(-7))
float_absolute = Fiddle::Function.new(libc["fabs"], [Fiddle::TYPE_DOUBLE], Fiddle::TYPE_DOUBLE)
p(float_absolute.call(-2.5))
single_absolute = Fiddle::Function.new(libc["fabsf"], [Fiddle::TYPE_FLOAT], Fiddle::TYPE_FLOAT)
p(single_absolute.call(-1.5))
power = Fiddle::Function.new(libc["pow"], [Fiddle::TYPE_DOUBLE, Fiddle::TYPE_DOUBLE], Fiddle::TYPE_DOUBLE)
p(power.call(2, 10))

process_id = Fiddle::Function.new(libc["getpid"], [], Fiddle::TYPE_INT)
p(process_id.call == Process.pid)

duplicate = Fiddle::Function.new(libc["strdup"], [Fiddle::TYPE_VOIDP], Fiddle::TYPE_VOIDP)
copied = duplicate.call("abc")
p([copied.class, copied.to_s, copied.to_s(2), copied.null?])
release = Fiddle::Function.new(libc["free"], [Fiddle::TYPE_VOIDP], Fiddle::TYPE_VOID)
p(release.call(copied))

# A String handed to a pointer argument is a buffer the function writes into.
buffer = "\0".b * 16
copy = Fiddle::Function.new(libc["strcpy"], [Fiddle::TYPE_VOIDP, Fiddle::TYPE_VOIDP], Fiddle::TYPE_VOIDP)
copy.call(buffer, "written")
p(buffer[0, 8])
p(buffer.bytesize)

p([Fiddle::SIZEOF_LONG, Fiddle::SIZEOF_VOIDP, Fiddle::SIZEOF_INT, Fiddle::ALIGN_DOUBLE, Fiddle::TYPE_SIZE_T])
