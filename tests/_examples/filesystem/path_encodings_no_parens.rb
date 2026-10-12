require "tmpdir"

# The pieces of a path, and the paths the system answers for one, are
# written in the encoding the path was given in.
path = "/ledgers/2024/summary.txt".encode Encoding::EUC_JP
extension = File.extname path
p extension.encoding
pieces = File.split path
p pieces.map(&:encoding)
Dir.mktmpdir do |directory|
  File.write File.join(directory, "rates.txt"), "1"
  named = File.join(directory, "rates.txt").encode Encoding::EUC_JP
  resolved = File.realpath named
  p resolved.encoding
  pattern = File.join(directory, "*.txt").encode Encoding::EUC_JP
  found = Dir.glob pattern
  p found.map(&:encoding)
  # A path that cannot be written in the encoding asked for keeps its bytes,
  # labeled with that encoding, for `realpath`, and keeps its own encoding
  # for `realdirpath`.
  inner = File.join directory, "rates_あ"
  Dir.mkdir inner
  latin = ".".encode Encoding::ISO_8859_1
  resolved = File.realpath latin, inner
  p resolved.encoding
  resolved = File.realdirpath latin, inner
  p resolved.encoding
end

# A positional flag beside a `flags:` keyword is not read.
found = Dir.glob "*.none", :ignored, flags: File::FNM_DOTMATCH
p found
begin
  Dir.glob "*.none", :ignored
rescue TypeError => problem
  p problem.message
end
