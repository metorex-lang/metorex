source = <<~RUBY
  case value
  when "" then :empty
  when :"", '' then :blank
  else :other
  end
RUBY
tree = RubyVM::AbstractSyntaxTree.parse(source)
case_node = tree.children.last
p case_node.type
when_node = case_node.children.last
p when_node.type
p [when_node.first_lineno, when_node.first_column]
p when_node.children.last.type
p [when_node.children.last.first_lineno, when_node.children.last.first_column]
