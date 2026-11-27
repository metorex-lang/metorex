# An exception whose class writes its own backtrace is reported with it, and
# one whose backtrace answers something has no locations once raised.
class Quiet < ArgumentError
  def backtrace(*) = []
end

class Placed < StandardError
  def backtrace(*) = ["x.rb:3:in 'step'", "z.rb:1"]
end

p Quiet.new("m").full_message(highlight: false).sub(/\A.*?:\d+:/, "HERE:")
p Placed.new("m").full_message(highlight: false)
begin
  raise Quiet, "m"
rescue Quiet => error
  p error.backtrace_locations
  p error.backtrace
end
