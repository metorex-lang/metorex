def pick(value, flag)
  if value == :first
    :first
  elsif not String === value or flag
    :not_a_string_or_flagged
  else
    :plain
  end
end

p pick(7200, false)
p pick("text", false)
p pick("text", true)
p(not String === 1)
p((not true or false))
p(not 1 + 1 == 2)
p(not [1, 2].include? 3)
p((not false and false))
p((not nil.nil? ? :yes : :no))
p(not(true).to_s)
