space = " ".encode("UTF-16LE")
word = "carté".encode("UTF-16LE")
newline = "\n".encode("UTF-16LE")

joined = "#{space}#{word}"
p [joined.encoding, joined.encode("UTF-8")]

built = String.new("#{space}#{word.chomp}")
p [built.encoding, built.encode("UTF-8")]

built[0, 1] = "-".encode("UTF-16LE")
p built.encode("UTF-8")

list = [space + "a".encode("UTF-16LE"), space + "b".encode("UTF-16LE")]
list[1, 0] = "+".encode("UTF-16LE") + word
p list.map { |line| line.encode("UTF-8") }

together = list.join(newline)
p [together.encoding, together.encode("UTF-8")]

head = "@@ -1 +1 @@\n".encode("UTF-16LE")
head << together
p head.encode("UTF-8")

two = String.new("#{"  ".encode("UTF-16LE")}#{word.chomp}")
p [two.encoding, two.encode("UTF-8")]

p [word[0, 1].encode("UTF-8"), word[1..2].encode("UTF-8"), word[-1].encode("UTF-8")]
