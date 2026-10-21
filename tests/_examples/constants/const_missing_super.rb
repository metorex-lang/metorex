module Inventory
  def self.const_missing(name)
    return :found if name == :Widget

    super
  end
end

p Inventory::Widget
begin
  Inventory::Gadget
rescue NameError => error
  puts error.message
  p error.name
end
begin
  Inventory.const_missing(:Sprocket)
rescue NameError => error
  puts error.message
end
