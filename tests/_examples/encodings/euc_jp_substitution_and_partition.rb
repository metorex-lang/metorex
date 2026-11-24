y = "あい a$b".encode("EUC-JP")
p y.sub(/z/, "")
p y.sub(/a/, "A")
p y.sub("a", "A")
p y.gsub(/\s/, "_")
p y.partition(/ /)
p y.split(/ /)
p y[0]
p (y + "!").bytes
