arms = (1..3000).map { |number| "when #{number} then low = #{number}; high = #{number * 2}" }
source = "def pick(value)\n  case value\n#{arms.join("\n")}\n  end\n  [low, high]\nend\n"
eval(source)
p pick(2999)
p pick(0)
