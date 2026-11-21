require "stringio"
require "tmpdir"

class RecordingInput
  attr_reader :calls

  def initialize(text)
    @io = StringIO.new(text)
    @calls = []
  end

  def gets(*args, **keywords)
    @calls << [:gets, args, keywords]
    @io.gets(*args, **keywords)
  end

  def readline(*args, **keywords)
    @calls << [:readline, args, keywords]
    @io.readline(*args, **keywords)
  end

  def each_line(*args, **keywords, &block)
    @calls << [:each_line, args, keywords]
    @io.each_line(*args, **keywords, &block)
    self
  end

  def readlines(*args, **keywords)
    @calls << [:readlines, args, keywords]
    @io.readlines(*args, **keywords)
  end
end

saved = $stdin

$stdin = RecordingInput.new("a\nb\n")
stream = ARGF.class.new
p stream.gets(chomp: true)
p stream.readline("\n", 1)
p [stream.lineno, stream.filename, $FILENAME]
p $stdin.calls

$stdin = RecordingInput.new("a\nb\n")
lines = []
p ARGF.class.new.each_line("\n", 1, chomp: true) { |line| lines << line }.equal?(nil)
p lines
p $stdin.calls

$stdin = RecordingInput.new("a\nb\n")
p ARGF.class.new.readlines("\n", 1, chomp: true)
p ARGF.class.new.to_a
p $stdin.calls

$stdin = StringIO.new("a\n")
stream = ARGF.class.new
p stream.readline
begin
  stream.readline
rescue EOFError => error
  p error.message
end

only_gets = Object.new
def only_gets.gets(*) = "line\n"
$stdin = only_gets
begin
  ARGF.class.new.readline
rescue NoMethodError => error
  p error.name
end
$stdin = Object.new
[:gets, :readlines].each do |name|
  ARGF.class.new.public_send(name)
rescue NoMethodError => error
  p error.name
end
$stdin = saved

Dir.mktmpdir do |directory|
  first = File.join(directory, "first.txt")
  second = File.join(directory, "second.txt")
  File.write(first, "one.two\n")
  File.write(second, "three\n")
  p ARGF.class.new(first, second).each_line(".", 8, chomp: true).to_a
  collected = []
  ARGF.class.new(first, second).each_line(chomp: true) { |line| collected << line }
  p collected
end
