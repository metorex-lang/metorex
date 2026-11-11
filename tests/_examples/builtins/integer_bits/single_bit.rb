# One bit of a machine-word Integer, read at positions inside the word, past
# its top, and below its bottom.
negative = -6
largest = 9_223_372_036_854_775_807
p [5[0], 5[1], 5[2], 5[3]]
p [negative[0], negative[1], negative[2], negative[3], negative[63], negative[64], negative[200]]
p [largest[62], largest[63], largest[64]]
p [5[-1], negative[-1]]
p [5[2**64], negative[2**64]]

class Integer
  def [](bit)
    [:written, bit]
  end
end
p 5[0]
