# File.stat of a missing file names the path in its message, in the bytes the
# path was written in.
["/missing-file-for-stat", "/missing-file-for-stat\xE3E4".b].each do |path|
  File.stat(path)
rescue SystemCallError => error
  p error.class, error.message, error.message.encoding
end
