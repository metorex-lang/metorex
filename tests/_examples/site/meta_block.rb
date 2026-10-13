def capture(&block) = block

rule = capture { |order| order * 2 }
p rule.arity
p rule.statements.first
p rule.call(21)
