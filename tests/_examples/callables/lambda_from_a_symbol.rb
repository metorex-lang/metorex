# Kernel#lambda takes a symbol's proc, which is a lambda already, and still
# refuses a plain proc handed over with &.
upcased = lambda(&:upcase)
p upcased.lambda?
p upcased.call("a")
p({ a: 1 }.map(&:first))

def relay(&block)
  lambda(&block)
end
begin
  relay { 1 }
rescue ArgumentError => error
  p error.message
end
