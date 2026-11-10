# A keyword followed by = names a setter as a symbol, as :name= does, which
# is how nokogiri aliases next= and previous=.
class Node
  attr_reader :log

  def initialize = @log = []
  def add_next_sibling(node) = @log << [:next, node]
  def node_name=(name)
    @log << [:name, name]
  end

  alias_method :next=, :add_next_sibling
  alias_method :name=, :node_name=
end
node = Node.new
node.next = "sibling"
node.name = "entry"
p node.log
p %i[next= class= if= end= def= nil= redo= yield= then= begin=].map { |name| :"#{name}" == name }.all?
p [:next=, :class=, :if=, :end=, :nil=, :begin=]
p({ a: 1 } == { a: 1 }, :a == :a)
