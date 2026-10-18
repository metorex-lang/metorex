# Methods that call themselves past the depth Metorex allows, which raises
# SystemStackError rather than running out of the native stack.
def down(count)
  count.zero? ? 0 : down(count - 1) + 1
end

begin
  down(100_000)
rescue SystemStackError => error
  puts error.message
end

# A message no method answers, which method_missing sends again.
class Resender
  def method_missing(name, *arguments)
    send(name, *arguments)
  end

  def respond_to_missing?(*) = true
end

begin
  Resender.new.unanswered
rescue SystemStackError => error
  puts error.message
end

# method_missing passing the message to the one above it, which sends it
# again.
class Forwarder
  def method_missing(name, *arguments)
    super
  rescue NoMethodError
    public_send(name, *arguments)
  end
end

begin
  Forwarder.new.unanswered
rescue SystemStackError => error
  puts error.message
end

# The same inside a fiber and a thread, whose stacks are smaller than the
# program's.
def through_a_block(count, &block)
  count.zero? ? block.call : through_a_block(count - 1) { [block.call] }
end

puts(Fiber.new do
  through_a_block(100_000) { 0 }
rescue SystemStackError => error
  error.message
end.resume)

puts(Thread.new do
  through_a_block(100_000) { 0 }
rescue SystemStackError => error
  error.message
end.value)
