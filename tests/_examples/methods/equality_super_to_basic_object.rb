# An == that falls back to BasicObject#== through super and then compares
# by value, as prism's Ripper lexer state does.
class LexState
  attr_reader :to_int

  def initialize(value)
    @to_int = value
  end

  def ==(other) super or to_int == other end
end

state = LexState.new(2)
p state == state
p state == 2
p state == 3
p 2 == state
p state == LexState.new(2)
