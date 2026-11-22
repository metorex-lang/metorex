string = "パスタ".encode(Encoding::ISO_2022_JP)
[-> { "" << string }, -> { "".dup.concat(string) }, -> { "" << "ab".encode("UTF-16LE") },
 -> { "x" << "é".encode("ISO-8859-1") }, -> { "".b << "é" }, -> { "a" << string }].each do |appending|
  made = appending.call
  p [made.encoding, made.bytesize]
rescue => error
  p error.class
end
