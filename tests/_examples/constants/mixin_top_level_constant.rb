module Helpers
  def helped = :top_level
end

module Outer
  module Helpers
    def helped = :nested
  end

  class Widget
    include ::Helpers
  end

  class Gadget
    extend ::Helpers
  end
end

p [Outer::Widget.new.helped, Outer::Gadget.helped]
