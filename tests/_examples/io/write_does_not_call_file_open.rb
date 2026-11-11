require "tmpdir"
path = File.join(Dir.mktmpdir, "out.txt")
class << File
  alias_method :plain_open, :open
  def open(*args, **options, &block)
    puts "open called"
    plain_open(*args, **options, &block)
  end
end
p File.write(path, "abc")
p File.read(path)
