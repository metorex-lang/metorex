invalid = (+"# \x8F\xFF\n").force_encoding(Encoding::EUC_JP)
p invalid.chars.size
invalid.encode!("UTF-8", invalid: :replace, undef: :replace)
p invalid
p invalid.match?(/\A# �+\n\z/)
undefined = (+"# \x8E\xE0\n").force_encoding(Encoding::EUC_JP)
p undefined.valid_encoding?
p undefined.encode("UTF-8", invalid: :replace, undef: :replace)
begin
  undefined.encode("UTF-8")
rescue Encoding::UndefinedConversionError => error
  p error.message
end
