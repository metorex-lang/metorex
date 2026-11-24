def keywords
  yield 1, two: 3
end

def splatted(items)
  yield(*items)
end

def bare_splat(items)
  yield *items, last: true
end

def rocket
  yield "a" => 1
end

keywords { |one, two:| p [one, two] }
keywords { |*all| p all }
splatted([4, 5]) { |a, b| p [a, b] }
bare_splat([6, 7]) { |a, b, last:| p [a, b, last] }
rocket { |pairs| p pairs }
