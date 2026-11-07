e = (1..3).map
p e.class, e.with_index.to_a, (1...3).map.with_index(1) { |n, i| n * i }, e.size
p (1..3).collect { _1 * 2 }
table = { b: 2, a: 1 }
walk = table.map
p walk.class, walk.size, walk.sort_by { |key, _| key.to_s }.map { |key, value| "#{key}=#{value}" }
class Params < Hash
  def listed = map.sort_by { |pair| pair.first.to_s }.map! { |key, value| "#{key}:#{value}" }
end
held = Params.new
held[:z] = 1
held[:y] = 2
p held.listed, table.collect.with_index.to_a
