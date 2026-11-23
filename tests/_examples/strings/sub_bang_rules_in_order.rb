# Rules prepended to an array are tried newest first, and the loop stops at
# the first one whose sub! changes the word.
rules = []
[[/$/, "s"], [/s$/i, "s"], [/(?:([^f])fe|([lr])f)$/i, '\1\2ves']].each do |rule|
  rules.prepend(rule)
end

%w[wives wife wolf cat].each do |word|
  result = word.dup
  rules.each { |(rule, replacement)| break if result.sub!(rule, replacement) }
  p result
end

p "wives".sub!(/s$/i, "s")
p "wives".sub!(/(?:([^f])fe|([lr])f)$/i, '\1\2ves')
