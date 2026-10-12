# `exception:` takes true or false and nothing else.
[-> { Integer("12", exception: 1) }, -> { Float("1.5", exception: nil) },
 -> { Rational("1/3", exception: :no) }, -> { system("true", exception: "yes") }].each do |call|
  begin
    call.call
  rescue ArgumentError => problem
    p problem.message
  end
end
p Integer("oops", exception: false)

# Integer() reads an object that spells itself as a String.
amount = Object.new
def amount.to_str
  "42"
end
p Integer(amount)
begin
  Integer(98, 15)
rescue ArgumentError => problem
  p problem.message
end

# `caller` refuses a negative start or length, and cuts a Float to a whole
# number.
[-> { caller(-1) }, -> { caller(0, -1) }].each do |call|
  begin
    call.call
  rescue ArgumentError => problem
    p problem.message
  end
end
p caller(0.9, 1.9).size
depth = Object.new
def depth.to_int
  0
end
p caller(depth).size
p caller(..0).size
p caller(-1..).size
p caller(100..)
p caller(100)

# A format that ends inside a directive is malformed, and one that leaves
# values unread says so when $VERBOSE is on.
["%", "hello %1$", " % "].each do |format|
  begin
    format(format, 1)
  rescue ArgumentError => problem
    p problem.message
  end
end
module Warning
  def self.warn(message, category: nil)
    print "warned: ", message.split("warning: ").last
  end
end
$VERBOSE = true
p format("%d", 1, 2)
