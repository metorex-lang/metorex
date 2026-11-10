# Eval coverage counts code from the line it is said to start on, line 0
# included, as ActionView compiles a template from line 0 so the `def` it
# wraps the template in is counted on no line; and the counts run through the
# code's last line, as a file's do.
require "coverage"
Coverage.start(lines: true, eval: true)
class View; end
View.module_eval("def render_show(items)\n  out = []\n  items.each { |i| out << i }\n  out\nend\n", "/app/show.html.erb", 0)
View.module_eval("def render_orphan\n  :never\nend\n", "/app/orphan.html.erb", 0)
View.module_eval("x = 1\ny = 2\n", "/app/plain.rb", 5)
eval("x = 1\n\ny = 2\nif x\n  y\nend\n", nil, "/app/evaluated.rb", 1)
View.new.render_show([1, 2, 3])
result = Coverage.result
p result["/app/show.html.erb"], result["/app/orphan.html.erb"], result["/app/plain.rb"], result["/app/evaluated.rb"]
