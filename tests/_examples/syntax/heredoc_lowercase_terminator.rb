# A heredoc's terminator may be written in lowercase, as Ruby's own test
# helpers write `<<eom`, with interpolation that reads a local of its own.
# After `class`, `<<self` still opens the singleton class.
line = 10
body = "p :run"
src = <<eom
# line #{line += 1}
#{body}
eom
p src
shifted = [1] << 2
p shifted

class Ledger
  class <<self
    def opened = :singleton
  end
end
p Ledger.opened
