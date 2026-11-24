require "random/formatter"

class CountingSource
  include Random::Formatter

  def bytes(count)
    "\x01".b * count
  end
end

source = CountingSource.new
p source.random_bytes.bytesize
p source.random_bytes(4)
p source.hex(2)
