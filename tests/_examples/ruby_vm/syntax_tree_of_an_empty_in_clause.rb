source = <<~RUBY
  case data
  in String then data = data.split
  in Array if data.all? { _1 in Integer | Float }
  else
    raise TypeError
  end
RUBY
tree = RubyVM::AbstractSyntaxTree.parse(source)
case_node = tree.children[2]
p case_node.type
clause = case_node.children[1]
p clause.type, clause.children.map { |child| child&.type }
second = clause.children[2]
p second.type, second.children.map { |child| child&.type }
p second.children[0].type, second.children[0].children.map { |child| child.class }
