key = "abc".encode("UTF-16LE")

begin
  key.split("/")
rescue Encoding::CompatibilityError => error
  puts error.message
end

p key.split("b".encode("UTF-16LE")).map { |piece| piece.encode("UTF-8") }

begin
  File.join("/storage", key[0..1], key)
rescue Encoding::CompatibilityError => error
  puts error.message
end
