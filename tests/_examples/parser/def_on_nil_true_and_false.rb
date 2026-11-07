def nil.described
  "nothing, #{inspect}"
end

def true.described = "yes"

def false.described
  "no"
end

p nil.described, true.described, false.described
p NilClass.instance_method(:described).owner
p nil.singleton_class
