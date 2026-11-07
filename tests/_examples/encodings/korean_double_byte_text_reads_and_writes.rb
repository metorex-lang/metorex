korean = "\xBD\xBA\xC6\xBC\xC7\xD8".b
p korean.dup.force_encoding("CP949").encode("UTF-8")
p korean.dup.force_encoding("EUC-KR").encode("UTF-8")
p "스티해".encode("CP949").bytes
p "\xB2\x41".b.force_encoding("CP949").encode("UTF-8")
p "\x81\x30".b.force_encoding("CP949").chars.size
p "\xB2\x41".b.force_encoding("EUC-KR").valid_encoding?
p korean.dup.force_encoding("EUC-KR").size
begin
  "\xB2".b.force_encoding("EUC-KR").encode("UTF-8")
rescue Encoding::InvalidByteSequenceError => problem
  p problem.message
end
