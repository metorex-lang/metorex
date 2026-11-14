# A class that makes a module's protected method public keeps it public for
# an object given a singleton class, as rspec-mocks gives one when it stubs
# a method, and a refused call names such an object as it inspects.
module Obligation
  protected

  def if_state(*states) = states
end
klass = Class.new do
  include Obligation
  public :if_state
end
held = klass.new
def held.synchronize = :stubbed
p held.if_state(:unscheduled), held.public_send(:if_state, :pending)
class Guarded
  protected def secret = 1
  private def hidden = 2
end
guarded = Guarded.new
def guarded.extra = 3
[:secret, :hidden].each do |name|
  message = begin
    guarded.public_send(name)
  rescue NoMethodError => error
    error.message
  end
  puts message.sub(/0x\h+/, "0x")
end
