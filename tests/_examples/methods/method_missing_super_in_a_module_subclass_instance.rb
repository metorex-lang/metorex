# `super` from a `method_missing` an instance of a Module subclass got from
# one included module reaches the one a module included before it defined.
module Answering
  def method_missing(name, *arguments)
    name == :pallet_count ? 12 : super
  end

  def respond_to_missing?(name, include_private = false)
    name == :pallet_count || super
  end
end

module Checking
  def method_missing(name, *arguments)
    @checked = name
    super
  end
end

module DoubleMethods
  include Answering
  include Checking
end

class ModuleDouble < Module
  include DoubleMethods
end

double = ModuleDouble.new
p double.pallet_count
begin
  double.crate_count
rescue NoMethodError => error
  puts error.message.sub(/0x\h+/, "0x")
end
