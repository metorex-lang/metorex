class Includer
  def include_in(klass, &customization)
    klass.class_exec(&customization)
  end
end

class Report
  def heading = :plain
end

group = Class.new(Report)
Includer.new.include_in(group) do
  undef :heading
  def heading = :custom
end
p group.new.heading
p Report.new.heading
p Includer.instance_methods(false)
