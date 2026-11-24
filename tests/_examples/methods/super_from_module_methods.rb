class Object
  def twin
    :object
  end
end
class Module
  def twin
    [:module, super]
  end
end
p Class.new.twin
p Module.new.twin
