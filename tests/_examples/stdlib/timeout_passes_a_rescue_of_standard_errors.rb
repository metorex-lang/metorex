# Inside the block, Timeout.timeout raises an exception that is not a
# StandardError, so a `rescue StandardError` in the code it stops does not
# catch it. It becomes a Timeout::Error at the Timeout.timeout call.
require "timeout"

def parse_or_nil
  loop {}
rescue StandardError
  :rescued_inside
end

begin
  Timeout.timeout(0.05) { parse_or_nil }
rescue Timeout::Error => error
  p([error.class, error.message])
end

begin
  Timeout.timeout(0.05) { loop {} }
rescue Exception => error
  p(error.class)
end

caught = nil
begin
  Timeout.timeout(0.05) do
    loop {}
  rescue Exception => inside
    caught = inside
    raise
  end
rescue Timeout::Error
  p([caught.class, caught.class.superclass, caught.message])
end

begin
  Timeout.timeout(5) { Timeout.timeout(0.05) { parse_or_nil } }
rescue Timeout::Error => error
  p([:inner, error.class])
end

# A class given to Timeout.timeout is raised as it is.
p(Timeout.timeout(0.05, ArgumentError, "slow") { parse_or_nil })

p(Timeout::Error.ancestors.include?(StandardError))
