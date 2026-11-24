blank = /\A[[:space:]]*\z/
%w[UTF-16BE UTF-16LE UTF-32BE UTF-32LE].each do |name|
  space = " ".encode(name)
  begin
    p blank.match?(space)
  rescue Encoding::CompatibilityError => error
    p error.message
  end
  encoded = Regexp.new(blank.source.encode(name), blank.options | Regexp::FIXEDENCODING)
  p [encoded.encoding, encoded.match?(space), encoded.match?("a".encode(name)), encoded.match?(" \t ".encode(name))]
end
wide = Regexp.new("b".encode("UTF-16LE"), Regexp::FIXEDENCODING)
p [wide.match?("ab".encode("UTF-16LE"), 1), wide.match?("ab".encode("UTF-16LE"), 2)]
