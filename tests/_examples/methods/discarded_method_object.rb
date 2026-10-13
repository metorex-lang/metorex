class Ledger
  def record(entry, amount = 0) = [entry, amount]
end

class Holder
  def initialize(object) = @object = object
  def handle = @object.method(:record)
  def handle_proc = proc { |required| required }
  def handle_lambda = ->(required) { required }

  def discards_a_method
    handle
    :done
  end

  def discards_a_proc
    handle_proc
    :done
  end

  def discards_a_lambda
    handle_lambda
    :done
  end
end

holder = Holder.new(Ledger.new)
p holder.discards_a_method
p holder.discards_a_proc
p holder.discards_a_lambda

def greet = puts("hello")

greet
held = method(:greet)
held
puts "a Method held in a local is not called"
