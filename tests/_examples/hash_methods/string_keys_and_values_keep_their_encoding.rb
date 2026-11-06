# A String key keeps the encoding it had when stored, and a value stored by
# reference shows a later force_encoding.
name = +"text"
name.force_encoding(Encoding::US_ASCII)
body = +"contents"
params = {}
params[name] = body
body.force_encoding(Encoding::US_ASCII)
p params["text"].encoding
p params.keys.map(&:encoding)
binary = {"x".b => 1}
p binary.keys.first.encoding
p binary.key?("x")
