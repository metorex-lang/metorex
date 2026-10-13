def grade(score)
  if score >= 90
    "A"
  elsif score >= 80
    "B"
  else
    "C"
  end
end

def label(value)
  value.positive? ? "plus" : "minus"
end

def describe_size(count)
  case count
  when 0
    "none"
  when 1..3
    "few"
  else
    "many"
  end
end

def unless_zero(value)
  unless value.zero?
    value * 2
  end
end

def countdown(from)
  steps = []
  while from > 0
    steps << from
    from -= 1
  end
  steps
end

def safe_length(text)
  result = text&.length
  result || 0
end

grade(95)
grade(85)
label(1)
describe_size(2)
unless_zero(3)
countdown(2)
safe_length(nil)
