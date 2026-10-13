class Invoice
  def total(price, quantity)
    price * quantity + 5
  end
end

method = Invoice.new.get_source(:total)
puts "#{method.name} takes #{method.arity} arguments"

def show(node, depth = 0)
  detail = case node["type"]
           when "BinaryOp" then node["op"]
           when "Identifier" then node["name"]
           when "IntLiteral" then node["value"]
           end
  puts "#{"  " * depth}#{node["type"]} #{detail}"
  show(node["left"], depth + 1) if node["left"]
  show(node["right"], depth + 1) if node["right"]
end

show(method.body.first)
