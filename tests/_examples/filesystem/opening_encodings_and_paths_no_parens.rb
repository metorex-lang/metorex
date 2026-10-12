# A mode naming `-` as the internal encoding, or the external one again,
# names no internal encoding, and a block given to File.open with a Pathname
# answers what the block does, as Bundler reads its lockfile.
require "pathname"
require "tmpdir"

Dir.mktmpdir do |directory|
  path = File.join directory, "rates.txt"
  File.write path, "# -*- coding: utf-8 -*-\nnorth 12\n"
  File.open(path, "r:UTF-8:-") { |file| p [file.external_encoding, file.internal_encoding, file.readline] }
  File.open(path, "r:UTF-8:utf-8") { |file| p [file.external_encoding, file.internal_encoding] }
  listed = Pathname.new path
  p File.open(listed, "r:UTF-8", &:read)
  p File.open(listed) { |file| file.readlines.size }
end
