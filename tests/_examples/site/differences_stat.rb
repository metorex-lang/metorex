stat = RubyVM.stat
p stat.slice(:global_cvar_state, :shape_cache_size).values.all?(&:zero?)
p stat.key?(:units_compiled) && stat[:instructions_compiled] > stat[:units_compiled]
p stat.fetch(:method_cache_hits, 0) > 0
