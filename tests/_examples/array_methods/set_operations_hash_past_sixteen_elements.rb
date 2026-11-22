poison = Object.new
poison.define_singleton_method(:hash) { raise "#hash called" }

def report(label)
  result = yield
  puts "#{label}: #{result.size}"
rescue RuntimeError => error
  puts "#{label}: #{error.message}"
end

report("& at 16") { Array.new(16) { Object.new } & Array.new(16, poison) }
report("& at 17") { Array.new(17) { Object.new } & Array.new(16, poison) }
report("| at 16") { Array.new(15) { Object.new } | [poison] }
report("| at 17") { Array.new(16) { Object.new } | [poison] }
report("- at 16") { Array.new(17) { Object.new } - Array.new(16, poison) }
report("- at 17") { Array.new(17) { Object.new } - Array.new(17, poison) }
report("& empty") { [] & Array.new(20, poison) }

numbers = (1..20).to_a
puts (numbers & [3, 3.0, 20, 1, 21]).inspect
puts (numbers | [3, 21, 21.0, 21]).last(3).inspect
puts (numbers - (5..30).to_a).inspect
