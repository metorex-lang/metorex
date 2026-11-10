# A method may share its name with a constant, as nokogiri's Nokogiri method
# does with its module: the bare name reads the constant, and a call with
# arguments reaches the method, whichever was defined first.
module Ledger
  def self.uses_gumbo? = true
  def self.parse(text) = "parsed #{text}"
end

def Ledger(*args)
  Ledger.parse(*args)
end

p Ledger.uses_gumbo?
p Ledger("entry")
p Ledger::VERSION rescue p $!.class
p(Ledger "line")

module Journal
  def self.name_of = :journal
end
p Journal.name_of
def Journal(*args) = [:method, args]
p Journal.name_of, Journal(1)
