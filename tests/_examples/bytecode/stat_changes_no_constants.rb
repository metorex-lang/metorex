# Reading RubyVM.stat changes no constant, so the constant caches it counts
# stay valid across the reads.
first = RubyVM.stat(:constant_cache_invalidations)
second = RubyVM.stat(:constant_cache_invalidations)
third = RubyVM.stat(:constant_cache_invalidations)
p [second - first, third - second]
