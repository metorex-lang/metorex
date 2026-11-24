class Node
  def inspect
    puts "inspected"
    "#<Node>"
  end
end

class Literal < String
end

[Node.new, "text", Literal.new("sql")].each do |node|
  kind = case node
         when Literal, ::String then :wrapped
         else :kept
         end
  p kind
end
p(Node.new.then { |node| String === node })
