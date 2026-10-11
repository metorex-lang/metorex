# encoding: utf-8
# Under the C locale the text a program reads and writes is US-ASCII. A name
# the file system gives that US-ASCII cannot hold is tagged binary, a broken
# US-ASCII string shows its bytes, and a Symbol past ASCII is quoted.
require "tmpdir"

p Encoding.default_external
p Encoding.find("filesystem")
p STDIN.external_encoding
p ARGV.map(&:encoding)

Dir.mktmpdir do |root|
  File.write(File.join(root, "café.txt"), "")
  name = Dir.children(root).first
  p([name.encoding, name.valid_encoding?])
end

broken = "©".dup.force_encoding("US-ASCII")
p broken.inspect
p :"ê".inspect
p :"ê".inspect.encoding
