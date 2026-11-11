list = ["a"]
def list.include?(x) = :stubbed
p list.include?("-s")
opts = { only: 1 }
opts.singleton_class.send(:define_method, :reject) { |*a, &b| :stubbed_reject }
p opts.reject { |k, v| v }
def check(argv) = argv.include?("-s")
p check(list)
ARGV.singleton_class.send(:define_method, :include?) { |*a| :argv_stub }
p ARGV.include?("-s")
plain = [1, 2]
p plain.first, plain.include?(2), plain.count
