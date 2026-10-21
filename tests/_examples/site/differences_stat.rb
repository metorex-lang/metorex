stat = RubyVM.stat
kept = %i[global_cvar_state shape_cache_size]
p stat.slice(*kept).values.all?(&:zero?)
units = stat[:units_compiled]
p units && stat[:instructions_compiled] > units
p stat.fetch(:method_cache_hits, 0) > 0
