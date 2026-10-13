def grouped(node)
  case node["type"]
  when "BinaryOp" then "(#{grouped(node["left"])} #{node["op"]} #{grouped(node["right"])})"
  when "Grouped" then grouped(node["expression"])
  when "IntLiteral" then node["value"].to_s
  when "Identifier" then node["name"]
  end
end

puts grouped(parse("rate * hours + 40").first)
puts grouped(parse("rate * (hours + 40)").first)
