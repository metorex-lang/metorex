require "tmpdir"

# On macOS a path in another ASCII-compatible encoding reaches the system as
# the characters it spells, so a name written in Windows-31J finds the file
# written under the same name in UTF-8.
Dir.mktmpdir do |directory|
  named = File.join(directory, "rates_あ.txt")
  File.write(named, "12")
  shift_jis = named.encode(Encoding::Windows_31J)
  p File.exist?(shift_jis)
  p File.size(shift_jis)
  p File.read(shift_jis)
  moved = File.join(directory, "moved_あ.txt").encode(Encoding::Windows_31J)
  File.rename(shift_jis, moved)
  p File.file?(named)
  p File.basename(File.realpath(moved)).encoding
end
