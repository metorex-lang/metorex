# The constant lookups a loop of constant reads makes, at two lengths.
LIMIT = 1

def limits(times)
  results = []
  times.times { results << LIMIT }
  results.uniq
end

def counted
  before = RubyVM.stat(:constant_cache_misses)
  yield
  RubyVM.stat(:constant_cache_misses) - before
end

limits(1)
counted { limits(100) }
p counted { limits(200) } == counted { limits(300) }
