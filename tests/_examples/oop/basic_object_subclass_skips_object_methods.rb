# A class descending from BasicObject does not answer methods added to
# Object, so method_missing receives them.
class Object
  def added_to_object
    :object_answered
  end
end

class Stand < BasicObject
  def method_missing(name, *arguments)
    [:missing, name]
  end
end

module Extra
  def from_kernel_copy
    :module_answered
  end
end

class StandWithModule < BasicObject
  include ::Extra
  def method_missing(name, *arguments)
    [:missing, name]
  end
end

::Kernel.p Stand.new.added_to_object
::Kernel.p StandWithModule.new.from_kernel_copy
::Kernel.p StandWithModule.new.added_to_object
