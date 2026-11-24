class Eaching < Hash
  def each(&block)
    { z: 9, y: 8 }.each(&block)
    self
  end
end
held = Eaching.new
p held.entries
p held.count
p held.sort_by { |key, value| value }
p held.min_by { |key, value| value }
p held.sum([])
p held.to_a
p held.select { true }
