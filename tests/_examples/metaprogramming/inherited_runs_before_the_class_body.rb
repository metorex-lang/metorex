# `inherited` runs when the class is made, before its body, so the body
# reads what the hook set up, as rss gives each element class its own list.
class Element
  ELEMENTS = []

  class << self
    def inherited(klass)
      klass.const_set(:ELEMENTS, [])
      super
    end

    def element(name) = self::ELEMENTS << name
  end
end

class Channel < Element
  element "title"
end

module Feeds
  class Item < Element
    element "link"
  end
end

p Element::ELEMENTS
p Channel::ELEMENTS
p Feeds::Item::ELEMENTS
