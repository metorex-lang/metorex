# A `do` inside parentheses in a condition opens a block on the call before
# it, in a `when` of a `case` too, as rubocop-ast's compiled node patterns
# write their matchers.
entries = [3, 8]
if (case
    when entries.any? do |entry|
      entry > 5
    end then true
    end)
  p :large_entry
end
if (true && entries.size.times do
  break :stopped
end == :stopped)
  p :stopped
end
while (false && entries.each do |entry| end)
end
p :done
