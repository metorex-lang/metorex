# A Regexp subclass's own methods answer before Regexp's, and `super`
# reaches the pattern it was built with.
class LoggedPattern < Regexp
  def match *arguments
    puts "match #{arguments.inspect}"
    super
  end

  def source
    "logged:#{super}"
  end
end

pattern = LoggedPattern.new "m."
found = pattern.match "meter"
p found[0]
p pattern.source

# String#match hands the work to the pattern.
found = "meter".match pattern
p found[0]

# A subclass instance built by `new` is not frozen, and `initialize` on it
# again refuses.
p pattern.frozen?
begin
  pattern.send :initialize, "x"
rescue TypeError => problem
  p problem.message
end
