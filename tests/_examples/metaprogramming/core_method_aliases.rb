# Ruby's core defines some methods twice under two names. A Method or an
# UnboundMethod for one name is equal to one for the other.
p Array.instance_method(:filter) == Array.instance_method(:select)
p Range.instance_method(:member?) == Range.instance_method(:include?)
p Kernel.instance_method(:kind_of?) == Kernel.instance_method(:is_a?)
p Kernel.method(:fail) == Kernel.method(:raise)
p Time.instance_method(:mday) == Time.instance_method(:day)
p Thread::SizedQueue.instance_method(:length) == Thread::SizedQueue.instance_method(:size)
p Struct.new(:width).instance_method(:values) == Struct.new(:depth).instance_method(:to_a)
p Array.instance_method(:size) == Hash.instance_method(:size)
p Array.instance_method(:filter) == Array.instance_method(:map)

# A class answers the methods Ruby defines on it, as Dir answers `pwd`.
p Dir.respond_to?(:getwd)
p Dir.method(:getwd) == Dir.method(:pwd)
p Thread.method(:fork) == Thread.method(:start)
p Kernel.public_methods(false).include?(:trap)
p Class.new.public_methods(false).include?(:superclass)

# An alias written in Ruby names the method it copies.
class Ledger
  def balance
    0
  end
  alias_method :total, :balance
end
p Ledger.instance_method(:total).original_name
# A method the program writes is not the core one of the same name.
p Ledger.instance_method(:balance) == Integer.instance_method(:abs)
p Ledger.instance_method(:total) == Ledger.instance_method(:balance)
