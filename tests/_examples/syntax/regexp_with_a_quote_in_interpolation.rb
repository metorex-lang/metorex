# A regexp written inside an interpolation, holding a quote or a brace, as
# RuboCop's autocorrections build their replacement strings.
def quote(text)
  "'#{text.gsub(/['\\]/) { |character| "\\#{character}" }}'"
end

def wrap(text)
  "<#{text.gsub(/[{"]/, "_")} #{text.length / 2} #{text.scan(/x/).size}>"
end

puts quote("it's")
puts wrap('{"x"}')
