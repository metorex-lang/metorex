candidates = []
"Nested::NestedUser".scan(/::|$/) { candidates.unshift "#{$`}::Child" }
p candidates

pieces = []
"a-b-c".scan(/-/) { pieces << [$`, $&, $'] }
p pieces
