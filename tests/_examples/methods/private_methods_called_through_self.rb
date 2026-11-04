# A call written with `self.` reaches a private method, a Kernel function
# among them, which a call through any other receiver is refused.

class Courier
  self.private
  def route = "inside"

  self.public def deliver = "#{self.route} #{self.format("%03d", 7)}"

  private_class_method :new
  def self.build = self.new

  self.puts "class body"
end

module Dispatch
  self.puts "module body"
end

courier = Courier.build
puts courier.deliver
p Courier.private_instance_methods(false)
p Courier.public_instance_methods(false)
begin
  Courier.new
rescue NoMethodError => error
  puts error.message
end
begin
  Object.new.puts "outside"
rescue NoMethodError => error
  puts error.message
end
