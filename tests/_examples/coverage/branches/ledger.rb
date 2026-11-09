def grade(score)
  if score > 90
    :high
  elsif score > 50
    :middle
  else
    :low
  end
end

def label(value)
  unless value
    :missing
  else
    value.to_s
  end
end

def size_of(kind)
  case kind
  when :small then 1
  when :large then 3
  end
end

[95, 60, 10, 20].each { |score| grade(score) }
[nil, 1, 2].each { |value| label(value) }
[:small, :large, :large, :other].each { |kind| size_of(kind) }
counter = 0
counter += 1 while counter < 3
note = nil
note&.strip
[1, nil].each { |value| value ? :present : :absent }
