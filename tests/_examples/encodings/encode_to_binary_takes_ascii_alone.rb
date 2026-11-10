# Converting text to BINARY takes ASCII and refuses any other character, as
# parser relies on to report `/あ/n` as an invalid regexp.
p(("あ".encode(Encoding::BINARY) rescue $!))
p(("é".force_encoding("ISO-8859-1").encode("BINARY") rescue $!))
p "abc".encode("BINARY").encoding, "é".encode("BINARY", undef: :replace)
p "\xff".b.encode("BINARY").bytes
