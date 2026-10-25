# A piece's to_s that changes a String joined before it leaves the joined
# text as it was when that String was taken.
class Appender
  def initialize(target)
    @target = target
  end

  def to_s
    @target << "!"
    "appended"
  end
end

taken = +"first"
p [taken, Appender.new(taken)].join("-")
p taken
p [:name, "text", 3].join(",")
