class Dog
  def initialize
    @attributes = {}
  end

  def _read_attribute(name)
    @attributes[name]
  end

  def _write_attribute(name, value)
    @attributes[name] = value
  end

  class_eval("def alias();self._read_attribute(\"alias\");end;def alias=(value);self._write_attribute(\"alias\", value);end;def alias?();!self.alias().nil?;end;def alias_was();self.alias();end")
  class_eval("def class?(); true; end;def end?();self.class?();end;def self.begin();:begun;end")
end

dog = Dog.new
p dog.alias?
dog.alias = "Rex"
p dog.alias
p dog.alias_was
p dog.end?
p Dog.begin

class Shape
  def class=(value)
    @kind = value
  end

  def end=(value)
    @finish = value
  end

  def summary
    [@kind, @finish]
  end
end

shape = Shape.new
shape.class = "circle"
shape.end = 9
p shape.summary
