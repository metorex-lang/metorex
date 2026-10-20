# Visibility a singleton class or a class gives a method Kernel answers
# natively holds for calls with a receiver and for `respond_to?`.
object = Object.new
object.singleton_class.send(:private, :frozen?)
p object.respond_to?(:frozen?)
begin
  object.frozen?
rescue NoMethodError => error
  puts error.message.sub(/0x\h+/, "0x")
end
p object.send(:frozen?)

class Ledger
  protected :hash
  def same_hash?(other) = other.hash == hash
end
ledger = Ledger.new
p ledger.same_hash?(ledger)
begin
  ledger.hash
rescue NoMethodError => error
  puts error.message.sub(/0x\h+/, "0x")
end

other = Object.new
other.singleton_class.send(:private, :frozen?)
other.singleton_class.send(:public, :frozen?)
p other.frozen?
