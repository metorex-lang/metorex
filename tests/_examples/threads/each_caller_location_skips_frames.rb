# Thread.each_caller_location skips as many frames as asked and stops at a
# length, as caller_locations does.
def innermost(*limits)
  labels = []
  Thread.each_caller_location(*limits) { |location| labels << location.label }
  labels.first(3)
end

def middle(*limits) = innermost(*limits)
def outer(*limits) = middle(*limits)

p outer
p outer(1)
p outer(2)
p outer(0, 2)
p outer(1..2)
p(outer(100))
