code = "def show\n  @name = 1\n  nil.upcase\nend\n"
view = Class.new
view.class_eval(code, "views/show.html.erb", -1)

begin
  view.new.show
rescue NoMethodError => error
  location = error.backtrace_locations.first
  p [location.path, location.lineno]
  id = RubyVM::AbstractSyntaxTree.node_id_for_backtrace_location(location)
  p id.is_a?(Integer)
end
