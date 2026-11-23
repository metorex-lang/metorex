require "date"

day = Date.new(2004, 2, 29).freeze
p day.frozen?
p [day.year, day.month, day.day]
p day.strftime("%Y-%m-%d")
moment = DateTime.new(2004, 2, 29, 10, 20, 30).freeze
p moment.strftime("%F %T")
