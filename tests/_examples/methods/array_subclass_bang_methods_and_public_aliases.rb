class Listing < Array
end
listing = Listing.new
listing << 1 << 2
p listing.map! { |item| item * 2 }.class
p listing.dup.map! { |item| item + 1 }.class
p listing.select! { |item| item > 2 }.class
p listing.concat([5]).class
p listing.push(6).class
p listing

class Picking < Hash
  def select(*args, &block)
    dup.tap { |held| held.select!(*args, &block) }
  end
  alias_method :filter, :select
end
picking = Picking.new
picking[:a] = 1
picking[:b] = 2
p picking.filter { |key, value| value > 1 }
p Picking.public_method_defined?(:filter)
