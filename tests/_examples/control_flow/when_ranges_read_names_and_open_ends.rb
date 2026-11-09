def place(level, limit)
  case level
  when limit + 1 then :past
  when 0..limit then :within
  when 0...limit then :short
  when 100.. then :far
  else :other
  end
end

p place(0, 1)
p place(2, 1)
p place(150, 1)
p place(-3, 1)
