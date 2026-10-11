module CharClassSubtraction
  refine Regexp do
    def -(other)
      /[#{source}&&[^#{other.source}]]/n
    end
  end
end

module ShiftMinutes
  refine Integer do
    def +(other)
      other.is_a?(String) ? "#{self} #{other}" : super(other)
    end
  end
end

def outside_the_refinement(left, right)
  left - right
rescue NoMethodError => error
  error.class
end

p(outside_the_refinement(/[a-z]/, /[aeiou]/))

using CharClassSubtraction
using ShiftMinutes

consonant = /[a-z]/ - /[aeiou]/
p(consonant)
p("rates"[consonant])
p(30 + "minutes")
p(30 + 15)
