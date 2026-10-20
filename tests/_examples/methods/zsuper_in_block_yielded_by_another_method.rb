# An implicit `super` inside a block passes the arguments of the method the
# block was written in, even while another method that took arguments of its
# own is the one yielding to the block.
class Label
  def text
    "crate"
  end

  def priced(amount)
    "#{amount} #{text}"
  end
end

class FragileLabel < Label
  def text
    framed("fragile") { super }
  end

  def priced(amount)
    framed("rush") { super }
  end

  def framed(word)
    "#{word}: #{yield}"
  end
end

puts FragileLabel.new.text
puts FragileLabel.new.priced(3)
