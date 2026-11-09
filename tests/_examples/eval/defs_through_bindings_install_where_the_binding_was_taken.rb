TOPLEVEL_BINDING.eval(<<~RUBY)
  def self.__capture__
    binding
  end
  $captured = __capture__
  class << self; undef __capture__; end
RUBY

group = Class.new do
  def helper; end

  $example = proc do
    $captured.eval("private")
    $captured.eval("def formatted_total\n:total\nend", "(console)", 1)
  end
end

group.new.instance_exec(&$example)
p TOPLEVEL_BINDING.receiver.method(:formatted_total).owner
p group.private_instance_methods(false)
