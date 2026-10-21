# Each single-byte encoding reads its high bytes as its own characters, and
# writes them back, as rexml reads ISO-8859-3. A byte the encoding leaves
# undefined is refused, or replaced when asked.
["ISO-8859-2", "ISO-8859-3", "ISO-8859-5", "ISO-8859-7", "KOI8-R", "Windows-1251", "IBM866", "macRoman"].each do |name|
  read = "\xE6".force_encoding(name).encode("UTF-8")
  p [name, read, read.encode(name).bytes]
end
p "ĉ".encode("ISO-8859-3").bytes
begin
  "\xA5".force_encoding("ISO-8859-3").encode("UTF-8")
rescue Encoding::UndefinedConversionError => error
  p error.message
  p error.error_char.bytes
end
begin
  "\xA5".force_encoding("ISO-8859-3").encode("ISO-8859-1")
rescue Encoding::UndefinedConversionError => error
  p error.message
end
p "a\xA5b".force_encoding("ISO-8859-3").encode("UTF-8", undef: :replace)
begin
  "ĉ".encode("ISO-8859-5")
rescue Encoding::UndefinedConversionError => error
  p error.message
end
