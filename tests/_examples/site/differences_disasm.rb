source = "total = 2 + 3"
iseq = RubyVM::InstructionSequence.compile(source)
puts iseq.disasm
