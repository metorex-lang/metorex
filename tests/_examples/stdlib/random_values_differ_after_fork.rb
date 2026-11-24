require "securerandom"
SecureRandom.hex
rand
readers = 2.times.map do
  reader, writer = IO.pipe
  fork do
    reader.close
    writer.write([SecureRandom.hex, rand, Random.new_seed, Random.urandom(4).unpack1("H*")].inspect)
    writer.close
  end
  writer.close
  reader
end
Process.waitall
values = readers.map { |reader| eval(reader.read) }
p values.transpose.map { |pair| pair.uniq.size }
p SecureRandom.hex.size
p SecureRandom.random_number(10).between?(0, 9)
p SecureRandom.random_number(1.5) < 1.5
p (3..5).cover?(SecureRandom.random_number(3..5))
p SecureRandom.uuid.size
p SecureRandom.random_number.between?(0, 1)
