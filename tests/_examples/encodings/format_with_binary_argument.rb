# A binary argument outside ASCII makes the formatted string binary, with
# its bytes as they were.
bytes = "\xFF".dup.force_encoding(Encoding::BINARY)
p Encoding.compatible?(Encoding::UTF_8, bytes)
formatted = sprintf("  %<0>s", :"0" => bytes)
p formatted, formatted.encoding, formatted.bytes
p format("%s|", bytes).encoding, (format("%s", bytes) + "x").bytes
p "#{bytes}".encoding, "a#{bytes}".bytes

# power_assert's inspection: a binary inspect result that UTF-8 cannot
# take is relabeled instead.
inspected = begin
  "#{bytes.encode(Encoding::UTF_8)}(#{bytes.encoding})"
rescue Encoding::UndefinedConversionError, Encoding::InvalidByteSequenceError => error
  p error.class, error.message
  bytes.dup.force_encoding(Encoding::UTF_8)
end
p inspected.bytes, inspected.valid_encoding?
line = sprintf("  %<0>s", :"0" => inspected)
stripped = begin
  line.rstrip
rescue ArgumentError, Encoding::CompatibilityError => error
  p error.class, error.message
  line.b.rstrip.force_encoding(line.encoding)
end
p stripped.bytes, stripped.encoding
message = ["  a.length", "  | |", "  | 1", stripped].join("\n")
p message.bytes.last(3), message.encoding, message == "  a.length\n  | |\n  | 1\n  \xFF".b
