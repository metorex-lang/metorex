# alias names the unary operators as -@, +@ and ~@, bare or as symbols,
# as Active Record does with `alias :-@ :deduplicate`.
class Balance
  def initialize(cents) = @cents = cents
  def negated = Balance.new(-@cents)
  def same = self
  def flipped = :flipped
  def to_s = "#{@cents}"
  alias :-@ :negated
  alias +@ same
  alias ~@ flipped
  alias_method :!, :flipped
end
p((-Balance.new(120)).to_s, (+Balance.new(45)).to_s, ~Balance.new(1), !Balance.new(1))
p Balance.instance_method(:-@).name, Balance.instance_method(:~).name
