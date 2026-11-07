p "ab\ncd\r\nef\r".gsub(/(?<!\r)\n|\r(?!\n)/, "N")
p "xa yb zcd".scan(/(?<=a|cd)\b/).size
p "a" * 4000 + "\n" == ("a" * 4000 + "\n").gsub(/(?<!\r)\n/, "\n")
p "STRASSE" =~ /(?<=strasse)/i
