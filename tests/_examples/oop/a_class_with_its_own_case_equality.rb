# A class that writes its own `self.===` answers `===` with it, as Kramdown's
# Boolean option type does.
class Boolean
  def self.===(other)
    FalseClass === other || TrueClass === other
  end
end
p [Boolean === true, Boolean === false, Boolean === "no"]
type = Boolean
default = true
p !(type === default) && !default.nil?
p(case false when Boolean then :boolean else :other end)
