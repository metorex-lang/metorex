# A C extension reading digits with ruby_scan_digits and the digit tables,
# asking whether a string's encoding is ASCII compatible and whether its
# characters are clean, and starting itself through InitVM.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require(build_extension("c_digit_scanning.c", "c_digit_scanning", directory))
scanning = CDigitScanning.new

p(scanning.scan("12z", -1, 10))
p(scanning.scan("ff", 2, 16))
p(scanning.scan("123", 2, 10))
p(scanning.scan("", 0, 10))
p(scanning.scan("99999999999999999999999", -1, 10)[2])
p(scanning.hexdigits)
p(%w[0 9 a z A Z _].map { |character| scanning.digit_value(character) })
p(%w[0 f F g].map { |character| scanning.hex_digit(character) })
p(scanning.literal_length)
p(scanning.ascii_compatible("é"), scanning.ascii_compatible("a".encode("UTF-16LE")))
p(scanning.clean("abc"), scanning.clean("é"), scanning.clean("\xff".dup.force_encoding("UTF-8")))
p(scanning.character(0xe9))
report { scanning.size_overflow }
