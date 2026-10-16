# One method whose body compiles, and one whose body falls back to the tree
# walker. Neither result is printed, since printing runs code of its own.
def nothing
end

def answer
  while false
  end
  42
end

nothing
answer
