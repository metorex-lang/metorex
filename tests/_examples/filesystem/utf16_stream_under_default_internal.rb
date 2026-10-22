require "tmpdir"

Encoding.default_internal = "UTF-8"
Dir.mktmpdir do |dir|
  path = File.join(dir, "notes.xml")
  File.binwrite(path, "\xFF\xFE".b + "<a>b</a>".encode("UTF-16LE").b)
  separator = ">".encode("UTF-16LE")

  File.open(path) do |file|
    p [file.external_encoding, file.internal_encoding]
    p file.read(2).bytes
    file.binmode
    file.set_encoding("UTF-16LE", "UTF-16LE")
    p [file.external_encoding, file.internal_encoding]
    line = file.readline(separator)
    p [line.encoding, line.bytes]
  end
end
