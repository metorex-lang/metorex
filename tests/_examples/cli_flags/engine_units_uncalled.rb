# The methods of engine_units.rb, defined and never called, so neither body
# is compiled.
def nothing
end

def answer
  load File.join(__dir__, "engine_units_fallback.rb")
end
