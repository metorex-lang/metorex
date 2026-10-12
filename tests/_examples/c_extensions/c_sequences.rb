# A C extension reaching the instruction sequence and heap functions the
# debug gem declares for itself: what a sequence says about its code, the
# sequences the heap holds, the size of an object, and how deep the debug
# inspector's frames sit. The extension is required by its `.so` name.
require "tmpdir"
require_relative "build_helper"

class Ledger
  def total(rates, scale = 1, *rest, tax:, fee: 0, **options, &block)
    rates.sum * scale
  end
end

def nested(depth)
  return CSequences.depths if depth.zero?

  nested(depth - 1)
end

directory = Dir.mktmpdir
built = build_extension("c_sequences.c", "c_sequences", directory)
require(built.sub(/\.\w+\z/, ".so"))

described = CSequences.describe(RubyVM::InstructionSequence.of(Ledger.instance_method(:total)))
p(described[0])
p(described[1] == File.expand_path(__FILE__))
p(described.drop(2))
adder = proc { |left, right| left + right }
p(CSequences.describe(RubyVM::InstructionSequence.of(adder)).values_at(0, 2, 3, 4, 5))

written_here = []
CSequences.each_iseq { |iseq| written_here << [iseq.label, iseq.first_lineno] if iseq.path == __FILE__ }
p(written_here.sort.uniq)
p(CSequences.first_iseq.class)

p(CSequences.memsize("ledger").positive?)
outer = nested(0)
inner = nested(3)
p(inner[0] - outer[0])
p(outer[1] - outer[2])
p(inner[1] - outer[1])

FileUtils.rm_rf(directory)
