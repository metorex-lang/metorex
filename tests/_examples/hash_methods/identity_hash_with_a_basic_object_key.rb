class Bare < BasicObject
  def initialize(foo) = @foo = foo
end
held = {}.compare_by_identity
key = Bare.new(1)
held[key] = :node
p held.key?(key), held.size
plain = {}
begin
  plain[key] = 1
rescue NoMethodError => error
  p error.message
end
