# A modifier condition may be a compound assignment, as Sass's parser
# writes `return unless key ||= interpolation`, and a value after a comma on
# the right of an assignment may assign in turn, as a sprockets test writes.
def map_pair(key = nil)
  return :none unless key ||= yield
  [key, :value]
end
p map_pair { nil }, map_pair { :read }, map_pair(:given) { :read }
cache = {}
p :stored unless cache[:entry] ||= nil
p cache
count = 0
p count while (count += 1) < 3
sources  = ["script.js"],
names    = ["hello", "console"]
p sources, names
first = 1, second = 2, 3
p first, second
