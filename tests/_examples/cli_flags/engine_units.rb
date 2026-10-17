# Two methods whose bodies compile, one of which loads a file whose top level
# is refused. Neither result is printed, since printing runs code of its own.
def nothing
end

def answer
  load File.join(__dir__, "engine_units_refused.rb")
rescue SystemStackError
end

nothing
answer
