# `m0` reads strict base64 and refuses anything else.

["AAAA", "AAA=", "", "AA=", "AA==\n", "AB==", "AAB=", "AA==AAAA", "A===", "===="].each do |text|
  begin
    p [text, text.unpack1("m0")]
  rescue ArgumentError => error
    p [text, error.message]
  end
end
