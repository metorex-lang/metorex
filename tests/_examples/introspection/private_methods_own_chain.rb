# Without ancestors, a class's private methods include those of its
# superclasses' singleton classes.
class Meter
  class << self
    private
    def calibrate_all; end
  end
end

class WaterMeter < Meter
  class << self
    private
    def flush_all; end
  end
end

p WaterMeter.private_methods(false).select { |name| name.end_with?("_all") }.sort

# A method defined in a `Module.new` block is public, and the block's
# visibility does not carry past it.
built = Module.new do
  private
  def hidden; end
end
built.module_eval { def shown; end }
p built.public_instance_methods(false)
p built.private_instance_methods(false)

# A module's own instance methods are not methods of the module object.
p(Module.new { def reading; end }.respond_to?(:reading))
