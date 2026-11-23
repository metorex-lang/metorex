# A class method written with `def self.name` is set aside under another
# name on the singleton class, replaced, and put back, as test helpers stub
# Time.now.
class Clock
  def self.now(zone = "UTC") = "real #{zone}"
end

[Clock, Time].each do |klass|
  singleton = klass.singleton_class
  singleton.alias_method :__stubbed_now, :now
  klass.define_singleton_method(:now) { |*| "stubbed" }
  p klass.now
  p klass.__stubbed_now.class
  singleton.alias_method :now, :__stubbed_now
  singleton.undef_method :__stubbed_now
  p klass.now.class
  p klass.respond_to?(:__stubbed_now)
end
p Clock.now("CET")

# The order minitest's stub takes: the replacement is undefined before the
# original is put back.
singleton = Time.singleton_class
singleton.alias_method :__outer_now, :now
singleton.define_method(:now) { "outer" }
singleton.alias_method :__inner_now, :now
Time.define_singleton_method(:now) { "inner" }
p Time.now
singleton.alias_method :now, :__inner_now
singleton.undef_method :__inner_now
p Time.now
singleton.undef_method :now
singleton.alias_method :now, :__outer_now
singleton.undef_method :__outer_now
p Time.now.class
