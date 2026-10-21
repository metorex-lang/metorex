class ParseFailure < StandardError
  attr_reader :position

  def initialize(message = nil, position: nil)
    super(message)
    @position = position
  end

  def ==(other)
    other.class == self.class && message == other.message && position == other.position
  end
end

first = ParseFailure.new("unexpected token", position: 19)
second = ParseFailure.new("unexpected token", position: 19)
p first == second, first != second, [first] == [second]
p first == ParseFailure.new("unexpected token", position: 20)
p StandardError.new("a") == StandardError.new("a")

module Mail
  class Parser
    Token = Struct.new(:symbol, :value)
  end
end
p Mail::Parser::Token.name
p Mail::Parser::Token.new(:SPACE, " ")
