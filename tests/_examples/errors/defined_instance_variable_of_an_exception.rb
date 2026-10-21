# An exception's instance variables, and an Array's, are defined once
# assigned, as rexml's ParseException reads its source through one.
class ParseFailure < RuntimeError
  def initialize(source)
    super("bad input")
    @source = source
  end

  def line = (@source[2] if defined? @source and @source)
  def missing = defined?(@other)
end

failure = ParseFailure.new([12, 0, 1])
p failure.line
p failure.missing
held = []
held.instance_variable_set(:@tag, :ledger)
p held.instance_eval { defined?(@tag) }
