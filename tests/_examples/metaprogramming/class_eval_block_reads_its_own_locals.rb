# A block run by class_eval or Class.new reads the locals it captured where
# it was written, not those of the method that runs it, as the block Action
# Pack hands to Active Support's `execute_hook`, whose own `options` is a
# Hash, reads its own `options`.
class Base; end

module Hooks
  def self.execute(base, options, block)
    base.class_eval(&block)
  end

  def self.build(base, options, block)
    Class.new(base, &block)
  end
end

def hook
  options = Struct.new(:permit_all_parameters).new(true)
  month = "2026-11"
  proc do
    month = "2026-12"
    def self.made = :made_here
    [options.permit_all_parameters, month]
  end
end

block = hook
p Hooks.execute(Base, { run_once: true }, block)
p Base.made
p Hooks.build(Base, { run_once: true }, block).made
counter = 0
Base.class_eval { counter += 1 }
p counter
