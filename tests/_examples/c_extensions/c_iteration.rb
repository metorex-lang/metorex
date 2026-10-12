# A C extension handing a block to the method its walk calls, calling the
# method it overrides with keywords, and reaching the module, encoding and
# IO functions Ruby's C API has for them.
require "tmpdir"
require "stringio"
require_relative "build_helper"

directory = Dir.mktmpdir
built = build_extension("c_iteration.c", "c_iteration", directory)
require(built)

p(CIteration.each_of([3, 5, 8]))
p(CIteration.pairs({ "rate" => 2 }))

class Ledger
  def total(*amounts, scale: 1)
    amounts.sum * scale
  end
end

class Account < Ledger
end
CIteration.define_super(Account, "total")
p(Account.new.total(2, 3, { scale: 10 }))
options = Object.new
def options.to_hash
  { scale: 100 }
end
p(Account.new.total(1, options))

module Audited
  def total(*amounts, **options)
    [:audited, super]
  end
end
CIteration.prepend(Ledger, Audited)
p(Ledger.new.total(4))
p(Ledger.ancestors.first(2))

module Rates
  OLD_RATE = 1
end
CIteration.deprecate(Rates, "OLD_RATE")
$stderr = StringIO.new
Warning[:deprecated] = true
Rates::OLD_RATE
p($stderr.string.include?("constant Rates::OLD_RATE is deprecated"))
$stderr = STDERR

p(CIteration.dummy?(Encoding::UTF_7))
p(CIteration.dummy?(Encoding::UTF_8))
p(CIteration.get_io($stdout) == $stdout)
p(CIteration.positive_pow(3, 40))
p(CIteration.timeout_error)

# With nothing above it, the call reaches `method_missing`, and the C
# method's block is passed on.
class Fallback
  def method_missing(name, *arguments, **options)
    [name, arguments, options]
  end
end
class Statement < Fallback
end
CIteration.define_super(Statement, "lookup")
p(Statement.new.lookup(1, { rate: 2 }))
class Runner
  def run(*, **)
    yield
  end
end
class Job < Runner
end
CIteration.define_super(Job, "run")
p(Job.new.run({}) { :given })
begin
  Job.new.run(nil) { :given }
rescue TypeError => problem
  p(problem.message)
end

name = File.join(directory, "rates.txt")
File.open(name, "w") { |io| p(CIteration.opening(io)) }
File.open(name, "a") { |io| p(CIteration.opening(io)) }
File.open(name, "r") { |io| p(CIteration.opening(io)) }

FileUtils.rm_rf(directory)
