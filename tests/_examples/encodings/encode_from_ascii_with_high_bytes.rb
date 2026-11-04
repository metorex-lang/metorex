text = "a\x80b\xffc".dup.force_encoding("US-ASCII")
p text.encode("UTF-8", invalid: :replace)
p text.encode("UTF-8", invalid: :replace, replace: "?")
begin
  text.encode("UTF-8")
rescue Encoding::InvalidByteSequenceError => error
  p error.message, error.error_bytes, error.source_encoding_name, error.destination_encoding_name
end
p "plain".dup.force_encoding("US-ASCII").encode("UTF-8").encoding
