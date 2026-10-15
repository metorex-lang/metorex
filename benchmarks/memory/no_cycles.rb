# The same 200,000 objects as cycles.rb, each holding an Integer in place
# of the lambda, so nothing refers back to the object.
class Listener
  def initialize
    @callback = 1
  end
end
200_000.times { Listener.new }
puts "done"
