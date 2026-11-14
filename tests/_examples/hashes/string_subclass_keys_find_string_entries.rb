# An instance of a String subclass finds the entry of the String it equals,
# as pdf-reader looks up content stream operators with its Token class,
# whatever `hash` the subclass writes.
class Token < String
  def initialize(value)
    super
  end
end
OPERATORS = { "BT" => :begin_text, "Tf" => :set_font }.freeze
p OPERATORS[Token.new("BT")], OPERATORS[Token.new("Tf")], OPERATORS[Token.new("ET")]
p Token.new("BT").hash == "BT".hash
by_token = { Token.new("Td") => :move }
p by_token["Td"], by_token.keys.first.class
counts = Hash.new(0)
[Token.new("q"), "q", Token.new("Q")].each { |name| counts[name] += 1 }
p counts
class Shouting < String
  def hash = upcase.hash
  def eql?(other) = other.is_a?(String) && upcase == other.upcase
end
p({ "MOVE" => 1 }[Shouting.new("move")])
p({ Shouting.new("move") => 1 }[Shouting.new("Move")])
