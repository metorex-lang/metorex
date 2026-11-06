# `!~` asks `=~` and answers the opposite, for nil, a String, a Symbol, and
# an object whose class defines no `=~` of its own.
headers = {}
p(headers["Connection"] !~ /close/i)
p("keep-alive" !~ /close/i)
p("close" !~ /close/i)
p(:closed !~ /close/)
p((Object.new !~ /x/ rescue $!.class))

class Matcher
  def =~(other)
    other == :yes ? 0 : nil
  end
end

p(Matcher.new !~ :yes)
p(Matcher.new !~ :no)
