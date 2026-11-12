# A block written in a default closes over the parameters, which hold what
# they are bound to once the defaults have run.
def returns_itself(handler = -> { handler })
  handler.call.equal?(handler)
end
p returns_itself

def reads_the_bound_value(count = (reader = -> { count }; 3))
  [count, reader.call]
end
p reads_the_bound_value

def reads_an_earlier_parameter(first, reader = -> { first })
  first = :changed
  reader.call
end
p reads_an_earlier_parameter(:given)

def keeps_a_passed_value(count = 3, reader = -> { count })
  reader.call
end
p keeps_a_passed_value(8)
