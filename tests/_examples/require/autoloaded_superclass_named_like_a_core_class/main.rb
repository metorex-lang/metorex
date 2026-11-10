# A superclass a module around the class autoloads is loaded from there, ahead
# of a top-level class of the same name, as temple's
# `class ArrayBuffer < Array` in its Generators module does.
module Generators
  autoload :Array, File.join(__dir__, "array")
  autoload :ArrayBuffer, File.join(__dir__, "array_buffer")
end

p Generators::ArrayBuffer.superclass
p Generators::ArrayBuffer.kind
