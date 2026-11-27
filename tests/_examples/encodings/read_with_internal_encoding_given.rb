path = File.join(__dir__, "utf8_city.txt")
Encoding.default_internal = "Shift_JIS"

text = File.read(path, external_encoding: Encoding::UTF_8, internal_encoding: Encoding::UTF_8)
p text.encoding
p text.chomp == "Zürich"

opened = File.open(path, "r:UTF-8:UTF-8", &:read)
p opened.encoding
