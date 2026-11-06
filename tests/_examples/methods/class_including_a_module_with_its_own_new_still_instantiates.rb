# A module whose own `self.new` picks a class to build, included in the
# classes it builds, leaves Class#new to those classes.
module Part
  def self.new(value)
    value.is_a?(Integer) ? NumberPart.new(value) : TextPart.new(value)
  end

  def described
    "#{self.class.name}(#{@value})"
  end
end

class NumberPart
  include Part

  def initialize(value)
    @value = value
  end
end

class TextPart
  include Part

  def initialize(value)
    @value = value
  end
end

p(Part.new(1).described)
p(Part.new("one").described)
p(NumberPart.new(2).described)
p(TextPart.respond_to?(:new))
