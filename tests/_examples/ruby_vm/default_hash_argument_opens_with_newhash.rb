class Holder
  def current_adapter(options = {})
    options
  end
end

iseq = RubyVM::InstructionSequence.of(Holder.instance_method(:current_adapter))
puts iseq.disasm.lines.find { |line| line.strip.start_with?("0000") }
