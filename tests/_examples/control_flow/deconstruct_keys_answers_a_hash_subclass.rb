StringKeyed = Class.new(Hash) do
  def key?(key)
    super(key.to_s)
  end

  def [](key)
    super(key.to_s)
  end
end

holder = Object.new
holder.define_singleton_method(:deconstruct_keys) do |keys|
  p keys
  answer = StringKeyed.new
  answer["a"] = 1
  answer["c"] = 3
  answer
end

result = case holder
         in { b: 1 } then :b
         in { a: 1, c: } then [:a, c]
         end
p result
matched = (holder in { a: Integer => found })
p [matched, found]
