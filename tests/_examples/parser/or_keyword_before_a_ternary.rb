# `or` binds more loosely than a ternary, so the ternary is its right side.
def expires(given, created, age)
  given or created && age ? created + age : nil
end

p expires(5, nil, nil)
p expires(nil, 10, 2)
p expires(nil, 10, nil)
p((false or true ? :ternary : :other))
p((nil and 1 ? :never : :also_never))

# `and` and `or` share one precedence and read from the left.
chained = (true or false and false)
p chained
p((false and true or true))
def keyword_chain
  true or false and false
end
p keyword_chain
