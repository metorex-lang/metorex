# Symbol literals loaded in loops, one in a file written in UTF-8 and one in
# a file written in binary, which machine code loads from the Symbol each
# literal made the first time.
require_relative "binary_symbols"

names = []
count = 0
while count < 50
  names << :plain << :"two words" << :"café"
  count += 1
end
p names.uniq
p names.first.equal?(:plain)
p names.last.encoding

binary = binary_symbols(50)
p binary.uniq.size
p binary.first.encoding
