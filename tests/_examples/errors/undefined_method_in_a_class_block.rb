group = Class.new
Object.const_set(:ReportGroup, group)
begin
  group.class_exec do
    contex "misspelled" do
    end
  end
rescue NoMethodError => error
  puts error.message
end
begin
  group.class_exec { contex }
rescue NameError => error
  puts error.message
end

class Delegating
  def self.method_missing(name, *arguments)
    super
  end
end
begin
  Delegating.class_exec { contex "misspelled" }
rescue NoMethodError => error
  puts error.message
end
begin
  Delegating.contex
rescue NoMethodError => error
  puts error.message
end
