class Script
  def method_missing(name, *args)
    super
  end
  def run = boom!
  def ask = ready?
  def plain = missing
end
[:run, :ask, :plain].each do |name|
  Script.new.send(name)
rescue NameError => error
  p [error.class, error.message]
end

class Fallback
  def method_missing(name, *args) = super
end

class Proxy
  def initialize(fallback) = @fallback = fallback
  def method_missing(name, *args) = @fallback.__send__(name, *args)
end

begin
  Proxy.new(Fallback.new).instance_eval { no_such_method }
rescue NameError => error
  p error.class
end
