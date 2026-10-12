# A TracePoint aimed at an instruction sequence hears the code the sequence
# runs, a `script_compiled` event reports the sequence of the code it
# compiled, and a trace's block runs with no frame between it and the code
# that fired the event.
class Ledger
  def total(rates)
    sum = rates.sum
    sum * 2
  end
end

sequence = RubyVM::InstructionSequence.of(Ledger.instance_method(:total))
heard = []
trace = TracePoint.new(:line) { |tp| heard << tp.lineno }
trace.enable(target: sequence)
Ledger.new.total([1, 2])
trace.disable
p(heard)

heard.clear
trace.enable(target: sequence, target_line: 8)
Ledger.new.total([1, 2])
trace.disable
p(heard)

compiled = []
TracePoint.new(:script_compiled) { |tp| compiled << tp.instruction_sequence }.enable do
  eval("def doubled(value)\n  [value].map { |held| held * 2 }\nend")
  eval("1", nil, "ledger_report.rb", 4)
end
compiled.each do |iseq|
  children = []
  iseq.each_child { |child| children << [child.label, child.first_lineno] }
  p([iseq.path.sub(__FILE__, "here"), iseq.absolute_path, iseq.label, iseq.first_lineno, children])
end

labels = nil
calls = TracePoint.new(:call) { |tp| labels ||= caller_locations(0, 3).map(&:label) }
calls.enable(target: Ledger.instance_method(:total)) { Ledger.new.total([1]) }
p(labels)
