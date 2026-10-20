# A copy of a module carries the module's own methods, and `undef` on the
# copy's singleton class removes one from the copy alone. So does a copy of a
# module made with Module.new and named afterwards.
module Settings
  def self.configuration = :configured
end

copy = Settings.dup
p copy.configuration
class << copy
  undef configuration
end
p copy.respond_to?(:configuration)
p Settings.configuration
p copy.singleton_class.inspect.sub(/0x\h+/, "0x")

Object.const_set(:Ledger, Module.new)
module Ledger
  def self.configuration = :ledger
end
ledger_copy = Ledger.dup
p ledger_copy.inspect.sub(/0x\h+/, "0x")
class << ledger_copy
  undef configuration
end
p ledger_copy.respond_to?(:configuration)
p Ledger.configuration

module Journal
  def self.configuration = :journal
end
Journal.singleton_class
journal_copy = Journal.dup
p journal_copy.singleton_class.inspect.sub(/0x\h+/, "0x")
class << journal_copy
  undef configuration
end
p journal_copy.respond_to?(:configuration)
p Journal.configuration
