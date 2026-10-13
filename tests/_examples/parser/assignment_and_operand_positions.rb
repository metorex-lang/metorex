class Holder
  attr_accessor :value
end
holder = Holder.new

def assign_in_return(holder)
  return holder.value = :returned unless holder.value
  :kept
end
p assign_in_return(holder)

count = 0
while holder.value = (count < 2) && :looping
  count += 1
end
p count

def pair(first, second) = [first.class, second.class]
p pair lambda {}, lambda {}

def chosen(flag)
  if flag
    :from_if
  end || :from_or
end
p [chosen(true), chosen(false)]

def from_case(value)
  case value
  when 1 then :one
  end || :none
end
p from_case(2)
