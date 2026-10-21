# `find_all` on a Hash is Enumerable's, answering an Array of pairs, where
# `select` and `filter` answer a Hash, as rexml's SAX2 parser reads it.
attributes = { "xmlns" => "a", "xmlns:x" => "b", "id" => "c" }
declarations = attributes.find_all { |name, _| name =~ /^xmlns(:|$)/ }
p declarations
p declarations.collect! { |name, value| [name[6..], value] }
p attributes.select { |name, _| name == "id" }
p attributes.filter { |name, _| name == "id" }
p attributes.filter_map { |name, value| value if name == "id" }
