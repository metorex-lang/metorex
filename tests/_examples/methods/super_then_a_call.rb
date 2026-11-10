# A bare super hands on the arguments the method was called with, and a
# call after it, written with . or &., is made on what it answers.
class Base
  def rows(kind) = kind == :none ? nil : [[kind, 1]]
end
class Ledger < Base
  def rows(kind)
    super.to_h
  end
end
class Safe < Base
  def rows(...)
    super&.to_h
  end
end
class Explicit < Base
  def rows(kind)
    super(kind)&.to_h
  end
end
p Ledger.new.rows(:paid), Safe.new.rows(:paid), Safe.new.rows(:none), Explicit.new.rows(:none)
