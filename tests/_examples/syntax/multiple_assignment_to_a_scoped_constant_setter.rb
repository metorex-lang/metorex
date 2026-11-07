class Holder
  class << self
    attr_accessor :zone
  end
end
old_zone, ::Holder.zone = Holder.zone, "UTC"
p [old_zone, Holder.zone]
a, Holder.zone = 1, "EST"
p [a, Holder.zone]
::Holder.zone, b = "CST", 2
p [Holder.zone, b]
