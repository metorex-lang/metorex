# module_function with names makes the module's instance copies private, even
# for a name Kernel also defines.
helpers = Module.new do
  def test() "hello" end
  module_function :test
end

# The receiver's method_missing passes to super, which still refuses the
# private method.
class Forwarder
  def method_missing(name, *args, &block)
    super
  end
  private :method_missing
end

target = Forwarder.new
target.extend(helpers)
p target.respond_to?(:test)
p helpers.private_instance_methods(false)
p target.send(:test)
begin
  target.test
rescue NoMethodError => error
  p error.class
  p error.name
end

# alias_method over an unrelated method warns in verbose mode.
$VERBOSE = true
$stderr = $stdout
Class.new do
  def first; end
  def second; end
  alias_method :second, :first
  alias_method :second, :first
end

# The alias keyword warns the same way, and aliasing a name to itself does not.
Class.new do
  def third; end
  def fourth; end
  alias fourth third
  alias_method :third, :third
end
