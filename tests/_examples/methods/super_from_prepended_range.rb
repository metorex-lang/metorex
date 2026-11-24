module Widened
  def include?(value)
    value.is_a?(Range) ? cover?(value) : super
  end
end
Range.prepend(Widened)
p (1..3).include?(1.5)
p ("a".."d").include?("c")
p (..2).include?(1)
p (1..5).include?(2..3)
p (1..).include?(7)
