require "date"

later = DateTime.new(2017, 1, 15, 23, 0, 1, "+01:00")
p later - DateTime.new(2017, 1, 15, 22, 0, 0)
p DateTime.new(2017, 1, 15, 22, 0, 0) - later
p later - Date.new(2017, 1, 15), Date.new(2017, 1, 16) - DateTime.new(2017, 1, 15, 12, 0, 0)
p Date.new(2017, 1, 16) - DateTime.new(2017, 1, 15, 12, 0, 0, "-06:00")
p DateTime.new(2017, 1, 15, 12, 0, 0, "+05:30") - DateTime.new(2017, 1, 15, 12, 0, 0, "-05:30")
p (later - 1).to_s, Date.new(2017, 1, 16) - Date.new(2017, 1, 1)

class StampedTime < Time
  def dst?
    super
  end
  alias isdst dst?
end
p StampedTime.at(0).utc.dst?, StampedTime.at(0).utc.to_a.size, StampedTime.at(0).utc.isdst
