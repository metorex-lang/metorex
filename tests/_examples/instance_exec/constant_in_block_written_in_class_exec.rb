# A block written inside a block that `module_exec` runs puts the classes
# and constants it defines where its code was written, at the top level,
# when it runs later through `instance_exec`, while a `def` in the outer
# block still defines a method on the class.
module Groups; end

class Base
  def self.examples = (@examples ||= [])
  def self.example(&body) = examples << body

  def self.describe(&body)
    group = Class.new(self)
    Groups.const_set(:Shipping, group)
    group.module_exec(&body)
    group
  end
end

group = Base.describe do
  def weight = 12

  example do
    class Pallet; end
    LIMIT = 40
    puts Pallet.name
    puts weight
  end
end

group.examples.each { |body| group.new.instance_exec(&body) }
p defined?(::LIMIT)
p Groups::Shipping.const_defined?(:Pallet, false)
