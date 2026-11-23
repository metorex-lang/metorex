# DateTime takes its arithmetic and comparisons from Date, so a method a
# program puts in place on Date reaches a DateTime too.
require "date"

p %i[+ - >> << <=> == === eql? hash next_day succ].map { |name| DateTime.instance_method(name).owner }

class Date
  alias_method :plus_before, :+
  def +(other)
    other == :tenth ? plus_before(Rational(1, 10)) : plus_before(other)
  end
end

moment = DateTime.new(2004, 11, 24, 1, 4, 44, "-05:00")
p moment + :tenth
p moment + 1
p moment - Rational(1, 24)
p moment - DateTime.new(2004, 11, 24, 0, 4, 44, "-05:00")
p Date.new(2004, 11, 24) + :tenth
p Date.new(2004, 11, 24) - Date.new(2004, 11, 20)
