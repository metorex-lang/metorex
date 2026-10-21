cases = {
  "joined" => [
    "X\xC2\xAD\xc2" \
      "\xaa\xce",
    "plain"
  ],
  "valid" => [
    "first " \
      "second",
    3
  ],
}
p cases["joined"][0].bytes.size, cases["joined"][0].valid_encoding?, cases["joined"][1]
p cases["valid"]
p ["a" "b", "c" \
  "d"]
mixed = "é" "\xff" "ü"
p mixed.bytes, mixed.encoding
joined = "X\xC2" \
  "\xAA\xCE\xB0"
p joined.valid_encoding?, joined.encoding, joined.bytes, joined.length
p joined.gsub(/ª/, "a")
