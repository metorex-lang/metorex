short = proc { 1 + 2 }
long = lambda do |a|
  a
end
stabby = ->(x) { x }
bare = Proc.new {
  1
}
[short, long, stabby, bare].each do |callable|
  p RubyVM::InstructionSequence.of(callable).to_a[4][:code_location]
end
