# A bare yield ends at the colon of a ternary, written at the end of the
# line or with a space after it.
def with_block_or(fallback, &block)
  block ?
    yield :
    fallback
end

def same_line(fallback)
  block_given? ? yield : fallback
end

p(with_block_or(1) { 2 })
p(with_block_or(1))
p(same_line(3) { 4 })
p(same_line(3))
