stat = RubyVM.stat
p stat.slice(:constant_cache_misses, :next_shape_id).values.all?(&:zero?)
p stat.key?(:units_compiled) && stat[:instructions_compiled] > stat[:units_compiled]
