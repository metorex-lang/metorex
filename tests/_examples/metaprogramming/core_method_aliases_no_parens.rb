# Ruby's core defines some methods twice under two names. A Method or an
# UnboundMethod for one name is equal to one for the other.
same = Array.instance_method(:filter) == Array.instance_method(:select)
p same
same = Range.instance_method(:member?) == Range.instance_method(:include?)
p same
same = Kernel.instance_method(:kind_of?) == Kernel.instance_method(:is_a?)
p same
same = Kernel.method(:fail) == Kernel.method(:raise)
p same
same = Time.instance_method(:mday) == Time.instance_method(:day)
p same
same = Thread::SizedQueue.instance_method(:length) == Thread::SizedQueue.instance_method(:size)
p same
same = Struct.new(:width).instance_method(:values) == Struct.new(:depth).instance_method(:to_a)
p same
same = Array.instance_method(:size) == Hash.instance_method(:size)
p same
same = Array.instance_method(:filter) == Array.instance_method(:map)
p same

# A class answers the methods Ruby defines on it, as Dir answers `pwd`.
p Dir.respond_to? :getwd
same = Dir.method(:getwd) == Dir.method(:pwd)
p same
same = Thread.method(:fork) == Thread.method(:start)
p same
listed = Kernel.public_methods false
p listed.include? :trap
listed = Class.new.public_methods false
p listed.include? :superclass

# An alias written in Ruby names the method it copies.
class Ledger
  def balance
    0
  end
  alias_method :total, :balance
end
p Ledger.instance_method(:total).original_name
# A method the program writes is not the core one of the same name.
same = Ledger.instance_method(:balance) == Integer.instance_method(:abs)
p same
same = Ledger.instance_method(:total) == Ledger.instance_method(:balance)
p same
