# A module prepended to an object's singleton class reaches the object's own
# singleton method through `super`.
module Suffixing
  def value
    :"#{super}_prepended"
  end
end

object = Object.new
def object.value = :original
object.singleton_class.send(:prepend, Suffixing)
p object.value
p object.singleton_class.ancestors.first(2).map { |held| held.to_s.sub(/0x\h+/, "0x") }
