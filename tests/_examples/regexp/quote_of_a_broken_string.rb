[["a.\x85b.", "US-ASCII"], ["a.\x80", "UTF-8"], ["\xff", "UTF-8"], ["\xff", "US-ASCII"], ["a b\x82\xa0", "Shift_JIS"], ["\x82.", "EUC-JP"], ["a.\xff", "ASCII-8BIT"]].each do |text, named|
  quoted = Regexp.quote(text.dup.force_encoding(named))
  p [quoted.bytes, quoted.encoding, quoted.valid_encoding?]
end
