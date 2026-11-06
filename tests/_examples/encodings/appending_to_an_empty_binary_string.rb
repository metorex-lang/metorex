# An empty String.new is binary, and what is appended to it decides its
# encoding: ASCII keeps it binary, and text past ASCII takes the text's.
require "stringio"
plain = String.new << "(%bad-params%)"
p plain.encoding
p StringIO.new(plain).external_encoding
accented = String.new << "café"
p accented.encoding
p accented.size
p (String.new("x") << "é" rescue $!.class)
reader, writer = IO.pipe
p reader.respond_to?(:size)
p StringIO.new("foo").size
