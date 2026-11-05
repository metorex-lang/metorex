# Regexp#match on nil answers nil without running its block, and the block
# goes no further than that call.
def taker(value, &block)
  [value, block.nil?]
end

def charset_of(header)
  found = nil
  /charset=([^;]+)/i.match(header) do |match|
    found = match[1]
  rescue ArgumentError
    nil
  end
  found
end

p taker(charset_of(nil))
p taker(charset_of("text/html; charset=utf-8"))
