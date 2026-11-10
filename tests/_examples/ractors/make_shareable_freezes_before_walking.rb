# Ractor.make_shareable freezes an object before reading its instance
# variables, so what the object's own `freeze` sets is made shareable too,
# as Parser::Source::Buffer's `freeze` fills in its line offsets.
class Buffer
  def initialize(source)
    @source = source
  end

  def freeze
    @line_begins ||= [0] + @source.each_char.with_index.select { |char, _| char == "\n" }.map { |_, at| at + 1 }
    super
  end
end

buffer = Buffer.new("a\nb\n")
Ractor.make_shareable(buffer)
p Ractor.shareable?(buffer), buffer.instance_variable_get(:@line_begins), buffer.instance_variable_get(:@line_begins).frozen?
