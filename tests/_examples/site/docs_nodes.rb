node = parse("total = items.sum(:price) if ready").first
p node["type"]
p node["condition"]

assignment = node["then_branch"].first
p assignment["target"]["name"]
p assignment["value"]["method"]
p assignment["value"]["arguments"]
p node["else_branch"]

p parse("case size when 1 then :one end").first
