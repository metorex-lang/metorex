p "é\xFF".scrub.bytes
p "b/\xC3\xA9\xFF.rb".b.force_encoding("UTF-8").scrub
p (+"\xB0\xA1\xFF").force_encoding("EUC-JP").scrub.bytes
p (+"a\x8F\xFF").force_encoding("EUC-JP").scrub.bytes
