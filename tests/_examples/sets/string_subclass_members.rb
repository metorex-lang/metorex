require "set"

class SqlLiteral < String
end

directions = [:asc, "asc", "DESC"].to_set
literal = SqlLiteral.new("asc")
p directions.include?(literal)
p({ "asc" => 1 }[literal])
p literal.eql?("asc")
p literal.hash == "asc".hash
p [literal].to_set.include?("asc")
p [literal].to_set.first.class
