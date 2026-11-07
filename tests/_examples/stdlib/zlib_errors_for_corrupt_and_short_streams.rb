require "zlib"

def outcome
  yield
rescue Zlib::Error => error
  "#{error.class}: #{error.message}"
end

raw = Zlib::Deflate.new(6, -15).then { |deflater| deflater.deflate("ledger line\n" * 40) + deflater.finish }
wrapped = Zlib::Deflate.deflate("ledger line\n" * 40)

p outcome { Zlib::Inflate.new(-15).inflate(nil) }
p outcome { inflater = Zlib::Inflate.new(-15); inflater << nil; inflater.finished? }
p outcome { inflater = Zlib::Inflate.new(-15); inflater.inflate(raw[0, 10]); inflater.inflate(nil) }
p outcome { inflater = Zlib::Inflate.new(-15); inflater.inflate(raw[0, 10]); inflater.finish }
p outcome { Zlib::Inflate.new.finish }
p outcome { Zlib::Inflate.new(-15).inflate("\x07\x00".b) }
p outcome { Zlib::Inflate.new(-15).inflate("\x01\x05\x00\x00\x00".b) }
p outcome { Zlib::Inflate.new.inflate("\x00\x00abc".b) }
p outcome { Zlib::Inflate.new.inflate("\x78\x9d\x00".b) }
p outcome { Zlib::Inflate.new.inflate(wrapped.byteslice(0, wrapped.bytesize - 1) + "\x00".b) }
p outcome { Zlib.inflate("\x78\x9c\x07".b) }
p outcome { Zlib.inflate(wrapped[0, 6]) }
p outcome { Zlib::Inflate.new(-15).inflate("\x05\xe0\x01\x00".b) }
p outcome { Zlib::Inflate.new(-15).inflate("\x05\x00\x00\x00\x00".b) }

finished = Zlib::Inflate.new
finished << wrapped
p outcome { finished << nil; finished.finished? }, finished.finish.bytesize
p outcome { inflater = Zlib::Inflate.new; [inflater.inflate(wrapped).bytesize, inflater.inflate(nil), inflater.inflate(nil)] }
