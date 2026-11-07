require "time"

["2020-W01-2", "2020-W01", "20W012", "2020W012", "2020-w10-7 10:00", "Week 2020-W01-2 ok", "20-W01"].each do |text|
  p [text, Date._parse(text), Date.parse(text).to_s]
end
p Date._parse("-W-3"), Date._parse("W05")
p Date.parse("-W-3").cwday, Date.parse("W05").cweek
p DateTime.parse("2020-W01-2T10:30:00+00:00").to_s
p((Time.parse("2020-W01-2") rescue [$!.class, $!.message]))
p Date._parse("2024-03-09")
