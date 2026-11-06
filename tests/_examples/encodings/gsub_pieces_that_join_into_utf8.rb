# Bytes that a substitution writes one at a time, each invalid alone, read
# as the characters they spell once joined.
joined = "caf%C3%A9".gsub(/%\h\h/) { [$&[1, 2]].pack("H2").force_encoding(Encoding::UTF_8) }
p joined
p joined.size
p joined.valid_encoding?
built = "".dup
built << "\xC3".dup.force_encoding("UTF-8") << "\xA9".dup.force_encoding("UTF-8")
p built.size
p built == "é"
broken = "\xBF".dup.force_encoding("UTF-8")
p((broken.upcase rescue $!.message))
p((broken.downcase rescue $!.message))
p((broken.dup.upcase! rescue $!.message))
p broken.upcase(:ascii)
