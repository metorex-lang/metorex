# `collect` with no block hands back an Enumerator, and `with_index` on it
# maps, as rexml numbers each attribute of an element.
class Attributes < Hash
  def each_attribute
    return to_enum(__method__) unless block_given?
    each_value { |value| yield value }
  end
end

held = Attributes.new
held["href"] = "mailto:a@b.c"
held["id"] = "first"
p held.each_attribute.collect.with_index { |value, i| [value, i + 1] }
p held.each_attribute.map.with_index(1) { |value, i| "#{i}:#{value}" }
p [3, 4].each.collect.with_index { |value, i| value * i }
