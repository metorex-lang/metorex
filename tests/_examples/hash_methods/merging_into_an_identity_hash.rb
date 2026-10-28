# Keys merged into a hash that compares by identity are found by identity,
# and keys merged out of one are found by value.

kinds = Hash.new(false)
kinds.compare_by_identity
kinds.update(:constant => "constant", :string => "string")
p kinds[:constant]
p kinds.key?(:string)

kinds.merge!(:keyword => "keyword") { |_key, held, given| held + given }
p kinds[:keyword]

kinds.update(:constant => "again") { |_key, held, given| "#{held} #{given}" }
p kinds[:constant]
p kinds.size

merged = kinds.merge(:comment => "comment")
p merged.compare_by_identity?
p merged[:comment]

by_value = { :plain => "plain" }.merge(kinds)
p by_value[:constant]
p by_value.size
