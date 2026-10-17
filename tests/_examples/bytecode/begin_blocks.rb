# `BEGIN` bodies run before the rest of the file, in the order written, and
# share its locals. `END` bodies run as the program ends, the last first.
puts "first statement"
BEGIN { order = [:begin_one] }
END { puts "end one" }
BEGIN { order << :begin_two }
END { puts "end two" }
p order
