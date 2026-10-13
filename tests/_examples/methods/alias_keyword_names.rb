class Condition
  def initialize(name) = @name = name
  def and(other) = Condition.new("#{@name} and #{other.name}")
  def or(other) = Condition.new("#{@name} or #{other.name}")
  def unless(other) = Condition.new("#{@name} unless #{other.name}")
  attr_reader :name

  alias & and
  alias | or
  alias except unless
  alias_method :both, :and
end

ready = Condition.new("ready")
paid = Condition.new("paid")
puts (ready & paid).name
puts (ready | paid).name
puts ready.except(paid).name
puts ready.both(paid).name
