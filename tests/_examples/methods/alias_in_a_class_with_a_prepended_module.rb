# An alias written in a class whose method a prepended module also defines
# names the module's method, so a new definition calling the alias comes
# back through the module until the stack runs out.
class Request
  def parse = [:original]
end
module Timing
  def parse = [:timed, *super]
end
Request.prepend(Timing)
class Request
  alias orig_parse parse
  def parse = [:secure, *orig_parse]
end
p Request.instance_method(:orig_parse).owner
begin
  Request.new.parse
rescue SystemStackError => error
  p error.class
end
