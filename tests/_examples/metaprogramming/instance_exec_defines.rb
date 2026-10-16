# A proc run with `instance_exec` on a module defines a method there and an
# alias of it.
DEFINITIONS = proc do
  def shout(text)
    text.upcase
  end
  alias yell shout
end

module Loud
  instance_exec(&DEFINITIONS)
end

p [Loud.instance_methods(false).sort, Loud.singleton_methods.sort, Loud.yell("a")]
