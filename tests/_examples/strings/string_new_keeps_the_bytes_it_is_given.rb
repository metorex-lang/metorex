p "Pok\xE9mon".bytes
p String.new("Pok\xE9mon").bytes
p String.new("Pok\xE9mon").valid_encoding?
p String.new("Pok\xE9mon", encoding: "BINARY").bytes
p String.new("\xE9".b).encoding
