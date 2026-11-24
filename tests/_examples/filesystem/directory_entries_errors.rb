require "tmpdir"
require "fileutils"
Dir.mktmpdir do |dir|
  file = File.join(dir, "foo.rb")
  FileUtils.touch(file)
  begin
    Dir.entries(file)
  rescue Errno::ENOTDIR => error
    p [:enotdir, error.class]
  rescue SystemCallError => error
    p [:other, error.class]
  end
  p File.join(*[dir, nil, "foo.rb"].compact) == file
  p File.join(*[nil, "foo.rb"].compact)
  queue = ::Queue.new
  queue << 1
  queue << 2
  taken = []
  taken << queue.pop until queue.empty?
  p taken
end
begin
  Dir.entries("/nonexistent-dir-for-metorex")
rescue SystemCallError => error
  p [error.class, error.message]
end
begin
  Dir.children(__FILE__)
rescue SystemCallError => error
  p [error.class, error.message.sub(__FILE__, "FILE")]
end
