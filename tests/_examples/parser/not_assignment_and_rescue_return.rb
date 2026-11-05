# `not` takes an assignment as its operand, and a rescue modifier's fallback
# ends at a comment.
def load_value(given)
  given
end

if not data = load_value(nil)
  p [:missing, data]
end
if not data = load_value(3)
  p :never
else
  p [:loaded, data]
end

def skips_on_failure
  value = Integer("x") rescue return # skip when the text is not a number
  value
end
p skips_on_failure

def falls_back
  Integer("y") rescue :fallback
end
p falls_back
