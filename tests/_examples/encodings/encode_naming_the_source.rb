bytes = "ren\xC3\xA9".b
relabeled = bytes.encode("utf-8", "utf-8")
p relabeled, relabeled.encoding, relabeled == "rené"
p bytes.encode("UTF-16LE", "UTF-8").bytesize
latin = "caf\xE9".b.encode("UTF-8", "ISO-8859-1")
p latin, latin.encoding
begin
  "\xFF".b.encode("UTF-8", "UTF-8")
rescue EncodingError => error
  p error.class, error.message
end
p "\xFF".b.encode("UTF-8", "UTF-8", invalid: :replace).encoding
