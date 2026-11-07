text = +"ab"
table = {}.compare_by_identity
table[text] = 1
text << "c"
p table[text]
text.upcase!
p table[text]
text.chomp!("C")
p table[text]
text.replace("other")
p table[text]
p table[+"other"]

list = [1]
table[list] = 2
list << 2
p table[list]

instance = Object.new
table[instance] = 3
instance.instance_variable_set(:@kept, 1)
p table[instance], table.size

before = Object.new.object_id
{}.compare_by_identity[Object.new] = 1
Marshal.load(Marshal.dump([[1], "a", { b: 2 }]))
p Object.new.object_id - before
