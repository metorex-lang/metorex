require "zlib"
require "digest"

text = (1..3000).map { |number| "entry #{number % 97} posted to account #{number * 7 % 131}\n" }.join
short = "quarterly totals: 12, 12, 12, 40, 12, 12\n" * 3

(-1..9).each do |level|
  wrapped = Zlib::Deflate.deflate(text, level)
  deflater = Zlib::Deflate.new(level, -Zlib::MAX_WBITS)
  raw = deflater.deflate(text) + deflater.finish
  p [level, wrapped.bytesize, wrapped.byteslice(0, 2).unpack1("H*"), level.zero? ? :stored : Digest::SHA256.hexdigest(wrapped)[0, 16],
     raw.bytesize, Zlib::Inflate.new(-Zlib::MAX_WBITS).inflate(raw) == text,
     Zlib::Deflate.deflate(short, level).unpack1("H*")[0, 40]]
end
p Zlib::Deflate.deflate("", 0).unpack1("H*"), Zlib.inflate(Zlib::Deflate.deflate("", 0))
big = "x" * 70_000
stored = Zlib::Deflate.deflate(big, Zlib::NO_COMPRESSION)
p stored.bytesize, Zlib.inflate(stored) == big
