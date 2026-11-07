HELD_CONSTANT = :first

def read_held = HELD_CONSTANT

p read_held
Object.send(:remove_const, :HELD_CONSTANT)
Object.const_set(:HELD_CONSTANT, :second)
p read_held

def read_environment = ENV

original = read_environment
Object.send(:remove_const, :ENV)
Object.const_set(:ENV, { "STUBBED" => "1" })
p read_environment.equal?(original)
p read_environment["STUBBED"]
Object.send(:remove_const, :ENV)
Object.const_set(:ENV, original)
p read_environment.equal?(original)
