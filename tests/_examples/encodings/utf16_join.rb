word = "carté".encode("UTF-16LE")
newline = "\n".encode("UTF-16LE")
edited = String.new("#{"  ".encode("UTF-16LE")}#{word}")
edited[0, 1] = "!".encode("UTF-16LE")
one = [edited].join(newline)
p [one.encoding, one.encode("UTF-8")]
two = [edited, word].join(newline)
p [two.encoding, two.encode("UTF-8")]
head = "--- 1 ----\n".encode("UTF-16LE")
head << one
p head.encode("UTF-8")
