class MarkedString < String
end

class Spelled
  def to_str
    "xy"
  end
end

text = +"hello"
text.bytesplice(0, 0, MarkedString.new("<b>"))
p text
text.bytesplice(0, 3, Spelled.new)
p text
marked = MarkedString.new("hello")
marked.bytesplice(5, 0, MarkedString.new("!"))
p marked, marked.class
begin
  text.bytesplice(0, 1, 5)
rescue TypeError => error
  p error.message
end
