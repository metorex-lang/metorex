class Buffer < String
  def to_s = self
end
class Plain < String; end
p "a-a".gsub(/a/) { Buffer.new("b") }
p "a-a".gsub(/a/) { Plain.new("c") }
p "a-a".sub(/a/) { Buffer.new("d") }
p "a-a".gsub("a") { Buffer.new("e") }
