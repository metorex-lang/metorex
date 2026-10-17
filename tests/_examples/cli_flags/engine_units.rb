# Two methods whose bodies compile, one of which loads a file whose top level
# falls back to the tree walker. Neither result is printed, since printing
# runs code of its own.
def nothing
end

def answer
  load File.join(__dir__, "engine_units_fallback.rb")
end

nothing
answer
