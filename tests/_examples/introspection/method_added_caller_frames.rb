class Recorder
  def self.method_added(name)
    puts caller(0, 3)
  end
end

class Named < Recorder
  def in_a_class_body
  end
end
