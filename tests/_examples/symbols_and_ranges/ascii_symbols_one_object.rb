# An ASCII Symbol is one object whatever encoding its name was spelled in.
utf8 = "plain".encode("UTF-8").to_sym
ascii = "plain".encode("US-ASCII").to_sym
binary = "plain".b.to_sym
p(utf8.equal?(ascii))
p(ascii.equal?(binary))
p(utf8.equal?(:plain))
p([utf8.encoding, ascii.encoding, binary.encoding])
p(utf8.object_id == ascii.object_id)
