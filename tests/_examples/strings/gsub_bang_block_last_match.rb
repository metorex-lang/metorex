# The block `gsub!` runs reads the match it was handed through `$~`, in a
# method and under a `while` modifier as anywhere else.
class Marker
  def mark(text)
    1 while text.gsub!(/\*(\w+)\*/) { |found| "<#{$~[1]}>" }
    text
  end
end

text = +"due *today* and *friday*"
text.gsub!(/\*(\w+)\*/) { $~.values_at(1).first.upcase }
p text
p Marker.new.mark(+"due *today*")

# `\K` leaves what the pattern read before it out of the match, and the
# block still reads the match.
text = +"Due *2026-11-01*"
text.gsub!(/(?:^|\W)\K(\*)([\w-]+?\S?)\1(?!\1)(?=\W|$)/) { "[#{$~[2]}]" }
p text
