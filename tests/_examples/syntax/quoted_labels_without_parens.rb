def take(*attributes)
  p attributes
end

take :karma, 'namespaced/email': true
take(:karma, 'namespaced/email': true)
take(:karma, "a b": 1, c: 2)
take 'x': 1, b: 2, 'c' => 3, "d e": 4
