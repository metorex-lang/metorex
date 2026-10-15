# A String holds bytes. Writing one byte, re-tagging, and packing into a
# buffer change the bytes and leave the encoding to read them.
accented = "é"
p accented.bytesize
p accented.getbyte(0)

accented.setbyte 1, 0xA8
p accented
p accented.length
p accented.encoding
p accented.valid_encoding?

retagged = "é".force_encoding "BINARY"
p retagged.bytes
p retagged.length
p retagged.force_encoding("UTF-8") == "é"

broken = "é"
broken.setbyte 0, 0xFF
p broken.valid_encoding?
p broken.bytes

buffer = "ab"
[0xC3, 0xA9].pack "CC", buffer: buffer
p buffer
p buffer.encoding

appended = +"a"
appended.append_as_bytes "\xC3".b, 0xA9
p appended
p appended.valid_encoding?
