soft = "first line=\n wraps\n\nsecond paragraph=\n"
decoded = soft.unpack1("M")
p decoded, decoded.encoding
p "first line=\r\n wraps\r\n=\r\n".unpack1("M")
p ["résumé café " * 6].pack("M")
p ["tab\there \nend"].pack("M"), ["x" * 80].pack("M").lines.map(&:size)
p "=41=42=3".unpack1("M"), "a=\r\nb".unpack1("M"), "bad=zz".unpack1("M")
p ["test \r \u{1F37F}"].pack("M")
p "=4".unpack1("M"), "x=".unpack1("M"), "a=zzb".unpack1("M"), "=41=4g".unpack1("M")
