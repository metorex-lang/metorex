# A loop of Integer operations that defines a method partway through, named
# for none of the operators.
def add_up
  total = 0
  count = 0
  while count < 400
    total += count
    count += 1
    Object.define_method(:helper) { nil } if count == 100
  end
  total
end

p add_up
