# Makes 200,000 objects that each hold a lambda referring back to the
# object, and keeps none of them. Each object and its lambda reference each
# other, so only a collector that finds cycles can free them.
class Listener
  def initialize
    @callback = -> { self }
  end
end

200_000.times { Listener.new }
puts "done"
