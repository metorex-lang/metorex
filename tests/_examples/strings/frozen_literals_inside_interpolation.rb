# frozen_string_literal: true

p "#{"".frozen?}"
p "#{"#{'a'.frozen?}"}"
p <<TEXT.chomp
#{"b".frozen?}
TEXT
p `echo #{"c".frozen?}`
p :"#{"d".frozen?}"
