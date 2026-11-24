module Mine
  def subclasses
    [:mine, super]
  end
  def name
    "named"
  end
end
class Class
  prepend Mine
end
class Parent; end
p Parent.subclasses
p Parent.name
p Class.ancestors.first(3)
