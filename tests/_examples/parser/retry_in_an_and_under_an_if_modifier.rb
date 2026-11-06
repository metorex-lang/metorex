# An if modifier covers the whole `a && retry` before it, so neither side
# runs when the condition is false.
def attempt(buffer)
  tries = 0
  calls = []
  begin
    tries += 1
    raise "again" if tries < 3
  rescue RuntimeError
    (calls << tries) && retry if buffer.empty?
  end
  [tries, calls]
end
p attempt("")
p attempt("held")
def plain(flag)
  ran = []
  (ran << :left) && ran << :right if flag
  ran
end
p plain(true)
p plain(false)
seen = []
[1, 2, 3, 4].each do |item|
  seen << item
  item.even? && next if item > 2
  seen << -item
end
p seen
p([1, 2, 3].map { |item| (next item * 10 if item.odd?); item })
def early(flag, other)
  other && return if flag
  :fell_through
end
p early(true, true)
p early(false, true)
p early(true, false)
count = 0
loop do
  count += 1
  count > 2 && break if count.odd?
end
p count
