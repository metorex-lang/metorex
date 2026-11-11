cache = Hash.new { |hash, key| hash[key] = key.to_s if key.is_a?(Symbol) }
p cache.values_at(*:keyword)
p cache.values_at(*[:a, :b])
p cache
p cache[:start_of_line], cache.values_at(:start_of_line)
