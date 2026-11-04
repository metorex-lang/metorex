# An instance of a String subclass matches a pattern as the string it holds.

class Line < String
  def indented? = self =~ /\A +/
end

line = Line.new("  item")
p line.indented?
p Line.new("item").indented?
p line =~ /item/
p line.send(:=~, /item/)
p line !~ /item/
p $~
p(/m\z/ =~ line)
begin
  line =~ "item"
rescue TypeError => error
  puts error.message
end
