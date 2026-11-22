require "tempfile"

file = Tempfile.new("foreach_and_readlines_open_keywords")
file.write("first\nsecond\n")
file.close

p IO.foreach(file.path, encoding: Encoding::BINARY).first.encoding
IO.foreach(file.path, mode: "rb") { |line| p line.encoding }
p IO.foreach(file.path, chomp: true).to_a
p IO.readlines(file.path, encoding: Encoding::BINARY).map(&:encoding).uniq
file.unlink
