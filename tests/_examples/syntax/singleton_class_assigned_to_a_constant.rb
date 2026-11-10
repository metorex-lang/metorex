# class << NAME = value stores the value under the name before the body
# runs, in a class body or a module body as at the top level, as haml's
# parser does with its AttributeMerger.
class Parser
  class << AttributeMerger = Object.new
    def merge(left, right) = left.merge(right)
  end
  private_constant :AttributeMerger

  def self.merged = AttributeMerger.merge({ id: 1 }, { class: "entry" })
end
p Parser.merged
p Parser.const_defined?(:AttributeMerger)
begin
  Parser::AttributeMerger
rescue NameError => error
  p error.message
end

module Ledger
  class << Totals = Object.new
    def sum(values) = values.sum
  end
end
p Ledger::Totals.sum([120, 300, 45]), Ledger.constants
