v = "M\xE9thode".b.dup.force_encoding(Encoding::UTF_8)
p v.valid_encoding?
p((v.unicode_normalize(:nfc) rescue [$!.class, $!.message]))
p(("ok".unicode_normalize(:nfd)))
p(("e\u0301".unicode_normalize(:nfc)))
