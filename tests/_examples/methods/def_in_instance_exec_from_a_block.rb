class Group
  def self.it(&block)
    (@blocks ||= []) << block
  end

  def self.blocks = @blocks
end

class Runner
  def around = yield

  def run(instance, block)
    around { around { instance.instance_exec(&block) } }
  end
end

Group.class_exec do
  it do
    eval %{def initialize(a:, b:); end}
  end
  it do
    def total = 5
  end
end

first = Group.new
Runner.new.run(first, Group.blocks[0])
second = Group.new
Runner.new.run(second, Group.blocks[1])
p([first.singleton_methods, second.singleton_methods, Group.instance_methods(false), Group.new.class])

object = Object.new
class << object
  def initialize = nil
end
p(object.singleton_methods)
