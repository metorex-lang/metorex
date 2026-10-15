# The methods every object answers through Kernel can be called without a
# receiver at the top level, as inside a method.
p(itself)
p(frozen?)
p(hash == self.hash)
p(object_id == self.object_id)
p(instance_variables)
def answers_itself = itself
p(answers_itself.equal?(self))

# `display` writes the object to $stdout, or to the port it is handed.
display
1.display
[1, :a].display
require "stringio"
port = StringIO.new
p("text".display(port))
p(port.string)
