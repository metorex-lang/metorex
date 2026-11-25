class TemplateFailure < StandardError; end

raise TemplateFailure.new("top") rescue p $!

found = begin
  raise "original"
rescue
  raise TemplateFailure.new("wrapped") rescue $!
end
p found

def rescued_in_method
  raise "original"
rescue
  raise TemplateFailure, "from method" rescue $!.message
end
p rescued_in_method
