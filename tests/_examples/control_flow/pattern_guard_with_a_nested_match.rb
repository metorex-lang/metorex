samples = [{"011" => 11}, 7, {"a" => "b"}, {"1" => 1, "2" => 2}]

samples.each do |value|
  case value
  in Integer
    puts "integer #{value}"
  in Hash if value.to_a in [[String => input, Integer => coerced]]
    puts "string #{input.inspect} coerces to #{coerced}"
  in Hash
    puts "other hash #{value.size}"
  end
end

matched = ({"7" => 7}.to_a in [[String => text, Integer => number]])
p matched
p text, number
