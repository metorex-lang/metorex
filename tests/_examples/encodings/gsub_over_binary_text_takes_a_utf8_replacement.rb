binary = "JkFPUS0mLSZBT1EtJi0mQU9RLQ==".unpack1("m")
p binary, binary.encoding

decoded = binary.gsub(/&([^-]+)?-/n) do
  if $1
    ($1.tr(",", "/") + "===").unpack1("m").encode(Encoding::UTF_8, Encoding::UTF_16BE)
  else
    "&"
  end
end
p decoded, decoded.encoding, decoded.valid_encoding?

later = "a&b".b.gsub("&") { "ä" }
p later, later.encoding

high = begin; "\xFF&".b.gsub("&") { "ä" }; rescue => error; error.class; end
p high

converted = "\x00\xE4".b.encode(Encoding::UTF_8, Encoding::UTF_16BE)
p converted.bytes
p "&".b.gsub("&") { converted }.bytes
p "&".b.gsub("&") { "ä" }.bytes
p "&-".b.gsub(/&-/n) { converted }.bytes
