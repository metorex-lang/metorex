def describe(value)
  case value
  when 1 then "one"
  when 2, 3 then "two or three"
  when :ready then "ready"
  when :ready, 4 then "four"
  when 5..9 then "five to nine"
  when :done then "done"
  end
end

def classify(code)
  case code
  when 14
    "newline"
  when 15
    "space"
  when 45
    "comment"
  else
    "other #{code.inspect}"
  end
end

p [1, 2, 3, :ready, 4, 7, :done, 10, nil].map { |value| describe(value) }
p [1.0, 3.0, 2r, "1", :other].map { |value| describe(value) }
p [14, 15, 45, 46, 14.0, :space].map { |code| classify(code) }

class Integer
  alias_method :__original_case_equal__, :===

  def ===(other)
    other == :every_integer || __original_case_equal__(other)
  end
end

p describe(:every_integer)
p classify(2**70)

statuses = %i[queued running finished]
counts = Hash.new(0)
200.times do |index|
  status = statuses[index % 3]
  label =
    case status
    when :queued then :waiting
    when :running then :busy
    when :finished then :idle
    end
  counts[label] += 1
end
p counts
