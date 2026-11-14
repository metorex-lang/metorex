# `clone` gives the copy the original's singleton class before
# `initialize_copy` runs, so a method stubbed on the original answers the
# copy's own calls there, as concurrent-ruby's MutableStruct spec counts its
# `synchronize` calls. `dup` leaves the singleton class behind.
class Locked
  def initialize = @values = [1]

  def initialize_copy(original)
    synchronize { super(original) }
  end

  protected

  def synchronize = yield
end
held = Locked.new
calls = []
held.singleton_class.class_eval do
  define_method(:synchronize) do |&block|
    calls << :stubbed
    block.call
  end
end
copy = held.clone
p calls, copy.singleton_methods
held.dup
p calls
