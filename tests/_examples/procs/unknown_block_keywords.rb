# A lambda or proc with keyword parameters and no `**rest` refuses a keyword
# it names no parameter for, as rubocop-ast's compiled node patterns rely
# on, after it has checked for missing ones.
matcher = ->(node, param:, block: nil) { [node, param] }
p((matcher.call(:int, param: 1, extra: 2) rescue $!))
tagged = proc { |entry, month:, year: 2026| [entry, month, year] }
p((tagged.call("rent", month: 11, day: 2, "note" => 3) rescue $!))
p((tagged.call("rent", day: 2) rescue $!))
p tagged.call("rent", month: 11), proc { |entry, month: 1, **rest| rest }.call("rent", day: 2)
def each_entry = yield("rent", day: 2)
p((each_entry { |entry, month: 1| month } rescue $!))
