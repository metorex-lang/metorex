# A double splat in an array literal spreads a Hash into the trailing one,
# which is left out when nothing was spread into it.
options = { size: 2 }
none = {}
p [1, **options]
p [1, **none]
p [**none]
p [*[1, 2], **options]
p [1, "kind" => :a, **options]
p [1, color: :red, **options]

def gathered(*args, **kwargs) = [args, **kwargs]
p gathered(1, 2)
p gathered(1, size: 3)
