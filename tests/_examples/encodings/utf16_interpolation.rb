plain = "items".encode("UTF-16LE")
accented = "itéms".encode("UTF-16LE")
space = " ".encode("UTF-16LE")
p "#{plain}".encoding
p "#{space}#{plain}".encoding
p "#{space}#{accented}".encoding
p String.new("#{space}#{plain}").encoding
