# Reading a date that leaves out its year or month falls back on the current
# day without calling `Date.today`, so a program that replaces it with a
# method taking no arguments, as timecop does, still parses dates.
require "date"
class Date
  class << self
    alias_method :today_without_mock_date, :today
    def today_with_mock_date
      today_without_mock_date
    end
    alias_method :today, :today_with_mock_date
  end
end
p Date.parse("Mar 3").month
p Date.parse("2020-01-02").to_s
p DateTime.parse("10:30").hour
p Date.strptime("12", "%d").day
p Date.today.is_a?(Date)
p Date.respond_to?(:current_day)
