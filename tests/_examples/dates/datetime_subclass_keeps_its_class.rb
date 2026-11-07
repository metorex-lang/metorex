require "date"

class StampedDateTime < DateTime
  attr_reader :stamp

  def stamped(value)
    @stamp = value
    self
  end
end

made = StampedDateTime.jd(2_451_545 + Rational(1, 4))
p made.class, made.to_s
p made.gregorian.class, made.julian.class, made.italy.class, made.england.class, made.new_start.class
p made.new_offset(Rational(1, 24)).class, made.new_offset(Rational(1, 24)).to_s
p (made + 1).class, (made - 1).class, (made >> 1).class, made.next_day.class
p StampedDateTime.civil(2024, 3, 9, 12, 30, 0).class
p StampedDateTime.new(2024, 3, 9).class, StampedDateTime.now.class
p made.gregorian.stamped(5).stamp
p StampedDateTime.parse("2024-03-09T10:00:00+00:00").class

class WorkDay < Date; end
day = WorkDay.new(2024, 1, 31)
p (day + 1).class, (day >> 1).class, (day >> 1).to_s, day.next_month.class, day.gregorian.class
p day.to_date.class, made.to_date.class, made.to_datetime.class
