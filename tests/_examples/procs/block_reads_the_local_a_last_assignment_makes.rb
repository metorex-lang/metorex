# A method's last statement may assign a local that a block on its right-hand
# side reads, which Ruby introduces ahead of the value, as Action View's
# with_empty_template_cache does with `subclass = Class.new(self) { ... }`.
class Base
  def self.with_empty_template_cache
    subclass = Class.new(self) {
      define_method(:compiled_method_container) { subclass }
      define_singleton_method(:compiled_method_container) { subclass }
    }
  end
end
made = Base.with_empty_template_cache
p made.compiled_method_container.equal?(made), made.new.compiled_method_container.equal?(made)

def countdown
  walk = ->(left) { left.zero? ? [:done] : [left] + walk.call(left - 1) }
end
p countdown.call(3)
