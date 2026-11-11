require 'forwardable'
class Conf
  extend Forwardable
  def_delegators :@hash, :[], :key?
  def initialize; @hash = { "a" => 1 }; end
end
c = Conf.new
def c.[](key) = { "stub" => key }
p c["a"], c.[]("a")
d = Conf.new
d.singleton_class.class_eval { define_method(:[]) { |*args| {} } }
p d["a"]
