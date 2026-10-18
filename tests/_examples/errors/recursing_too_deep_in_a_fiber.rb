# A fiber whose stack holds less than the program's, recursing through
# blocks, `instance_exec`, `public_send`, `method_missing`, and a Method
# object, each of which runs on the native stack. It raises SystemStackError
# before that stack runs out.
class Recurser
  define_method(:step) do |count|
    [count].map { |left| instance_exec(left) { |inner| public_send(:missing_step, inner) } }
  end

  def method_missing(name, count)
    return super unless name == :missing_step

    method(:step).call(count + 1)
  end

  def respond_to_missing?(*) = true
end

reached = Fiber.new do
  Recurser.new.step(0)
rescue SystemStackError => error
  [error.class, error.message]
end.resume
p(reached)
p(:carried_on)
