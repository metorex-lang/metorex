# A class written as `class Name < Struct.new(...)` inspects as a struct,
# alone and inside another struct, as net-imap's ResponseText does.
class ResponseText < Struct.new(:code, :text)
  EMPTY = new(nil, "").freeze
end

Response = Struct.new(:name, :data)
text = ResponseText.new(nil, "ready")
p text
p Response.new("OK", text)
p ResponseText::EMPTY

# A struct reached again while it is being inspected shows as its class.
Cell = Struct.new(:next_cell)
first = Cell.new
second = Cell.new(first)
first.next_cell = second
p first
