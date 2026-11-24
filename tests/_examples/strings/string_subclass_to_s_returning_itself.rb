class Buffer < String
  def to_s
    self
  end
end
held = Buffer.new("<br/>")
p "foo %{x} bar" % { x: held }
p "foo %s bar" % [held]
p "a#{held}b"
p [held].join("-")
