require "date"

module Outer
  module Date; end

  def self.kind(value)
    case value
    when ::Time, ::DateTime then :time
    when ::Date then :date
    when Date then :own
    else :other
    end
  end
end
p Outer.kind(::Date.new(2008, 3, 1)), Outer.kind(::DateTime.new(2008, 3, 1, 6)), Outer.kind(Time.now), Outer.kind(5)
p(::Date === ::Date.today, [::Date].include?(::Date))
module Outer
  def self.pattern(value)
    case value
    in ::Date then :top_date
    in ::Integer | ::Float then :number
    else :other
    end
  end
end
p Outer.pattern(::Date.new(2008, 3, 1)), Outer.pattern(4), Outer.pattern(:x)
