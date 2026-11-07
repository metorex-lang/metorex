held = String.new("Pok\xE9mon").force_encoding("BINARY")
p held.count("\x9\xA\xD\x20-\x3C\x3E-\x7E")
p held.count("a-z")
p held.count("^a-z")
p "a-c=x".count("\x20-\x3C\x3E-\x7E")
p held.bytes
p held.encoding
p [held].pack("m")
p held.valid_encoding?
p String.new("Pok\xE9mon").bytes
