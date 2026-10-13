class Guard < ::Mutex
  NEW_MUTEX_METHOD = Guard.method(:new)

  def self.new
    NEW_MUTEX_METHOD.call
  end
end
p Guard.new.class
p Guard.method(:new).arity
held = Mutex.method(:new)
p held.call.class
class Plain
  NEW = method(:new)
end
p Plain::NEW.call.class
