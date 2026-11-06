# frozen_string_literal: true
# A String constant written in a `class << self` body under the
# frozen_string_literal comment is frozen, and a Ractor can read it.
Warning[:experimental] = false
module Schemes
  class << self
    Reserved = ".+-"
    def reserved = Reserved
  end
end
p Schemes.singleton_class.const_get(:Reserved).frozen?
p Ractor.new { Schemes.reserved }.value
