# A bare `yield` is one side of a ternary, as rss's tests write
# `block_given? ? yield : content`, while `yield :total` still yields a Symbol.
def body(content = "empty")
  "<rss>#{block_given? ? yield : content}</rss>"
end

p body
p body { "items" }
choice = block_given? ? :given : :none
p choice

def tag = yield :total
p(tag { |name| name })
