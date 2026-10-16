# Each kind of assignment a body compiles, printed so the bytecode engine's
# answers can be compared with the tree walker's and Ruby's.

class Box
  attr_accessor :size, :label

  def initialize
    @size = 1
    @label = nil
  end
end

module Settings
  LIMIT = 3
  LIMIT_TWICE ||= LIMIT * 2
  Settings::NESTED = 4
  Settings::NESTED ||= 5
  ::ASSIGNMENT_TOP = 1
  ASSIGNMENT_TOP_TWICE ||= 2
end

def locals_and_operators
  count = 1
  count += 2
  count *= 3
  count -= 1
  count **= 2
  count %= 7
  flag = nil
  flag ||= :set
  flag &&= :replaced
  missing = false
  missing &&= :never
  [count, flag, missing, 7 <=> 3, 6 & 3, 6 | 3, 6 ^ 3, 2 ** 10]
end

def variables
  @held = 1
  @held += 1
  $assignment_global = 3
  $assignment_global ||= 4
  [@held, $assignment_global]
end

def receivers
  box = Box.new
  box.size = 5
  box.size += 1
  box.label ||= "first"
  box.label ||= "second"
  values = [1, 2, 3]
  values[0] = 10
  values[1] += 5
  values[0, 2] = [:a, :b]
  table = {}
  table[:count] ||= 0
  table[:count] += 1
  nothing = nil
  answered = (nothing&.size = 3)
  [box.size, box.label, values, table, answered, values[2], table[:count]]
end

def constants
  [Settings::LIMIT, Settings::LIMIT_TWICE, Settings::NESTED, ::ASSIGNMENT_TOP, Settings::ASSIGNMENT_TOP_TWICE]
end

def multiple
  first, second = 1, 2
  first, second = second, first
  head, *rest = [1, 2, 3]
  (inner, other), last = [4, 5], 6
  *start, final = 7, 8, 9
  box = Box.new
  box.size, box.label = 10, "ten"
  values = []
  values[0], values[1] = :x, :y
  answered = (one, two = 3, 4)
  [first, second, head, rest, inner, other, last, start, final, box.size, box.label, values, answered, one, two]
end

def chained
  a = b = 5
  [a, b, (c = 6) + 1, c]
end

p locals_and_operators
p variables
p receivers
p constants
p multiple
p chained
