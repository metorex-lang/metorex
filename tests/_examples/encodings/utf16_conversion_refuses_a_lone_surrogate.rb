decoded = " more up to 90+ACU===".unpack1("m")
p decoded.bytes

begin
  p decoded.encode(Encoding::UTF_8, Encoding::UTF_16BE)
rescue => error
  p error.class, error.message
end

p "\xD8\x00\xDC\x00".dup.force_encoding("UTF-16BE").encode("UTF-8").codepoints
p "\x00a\xDC\x00".dup.force_encoding("UTF-16BE").valid_encoding?
p "\x00\xD8".dup.force_encoding("UTF-16LE").encode("UTF-8", invalid: :replace)

[
  ["\x00a\xDC\x00\x00b", "UTF-16BE"],
  ["\x00a\xD8\x00\x00b", "UTF-16BE"],
  ["\x00a\xD8\x00", "UTF-16BE"],
  ["\x00a\x00", "UTF-16BE"],
  ["a\x00\x00\xDC", "UTF-16LE"],
  ["\x00\x00\x00a\x00\x11\x00\x00", "UTF-32BE"],
  ["\x00\x00\xD8\x00", "UTF-32BE"],
  ["\x00\x00\x00", "UTF-32BE"],
].each do |bytes, named|
  text = bytes.dup.force_encoding(named)
  begin
    text.encode("UTF-8")
  rescue => error
    p [error.class, error.message, error.error_bytes.bytes, error.readagain_bytes.to_s.bytes, error.incomplete_input?]
  end
  p text.encode("UTF-8", invalid: :replace)
end
