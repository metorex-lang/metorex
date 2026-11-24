class Plain < Hash
end
plain = Plain.new
plain[:a] = 1
p plain.entries
class Eaching < Hash
  def each(&block)
    { z: 9 }.each(&block)
    self
  end
end
p Eaching.new.entries
p Enumerable.instance_method(:entries).bind_call(Eaching.new)
p Enumerable.instance_method(:to_a).bind_call(Eaching.new)
p Eaching.new.to_a
p Eaching.new.map { |pair| pair }
