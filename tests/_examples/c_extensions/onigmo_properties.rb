# A C extension reading a Unicode property's Onigmo number and code point
# ranges through ruby/oniguruma.h, as regexp_property_values does.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require(build_extension("onigmo_properties.c", "onigmo_properties", directory))

hiragana = OnigmoProperties.number("Hiragana")
p(hiragana == OnigmoProperties.number("hira"), OnigmoProperties.number("Old_Italic") == OnigmoProperties.number("oldItalic"))
p(OnigmoProperties.ranges(hiragana))
p(OnigmoProperties.ranges(OnigmoProperties.number("ASCII_Hex_Digit")))
p(OnigmoProperties.number("No Such Property"), OnigmoProperties.number("Hiragänä"))
p(OnigmoProperties.ranges(100_000))
