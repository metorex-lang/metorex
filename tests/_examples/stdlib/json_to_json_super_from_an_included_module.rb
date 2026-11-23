# A module included into Object after json reaches the generator's to_json
# through super, for an object whose class has no to_json of its own.
require "json"

module ToJsonThroughSuper
  def to_json(state = nil)
    state.is_a?(::JSON::State) ? super(state) : "custom"
  end
end
[Enumerable, Object, Array, Hash, String, Integer].reverse_each { |klass| klass.include(ToJsonThroughSuper) }

p Time.instance_method(:to_json).owner
p Object.instance_method(:to_json).super_method&.owner
p Integer.instance_method(:to_json).super_method&.owner
p JSON.generate([:sym, 1..2, /x/, 1, "s", { "k" => :v }])
p :sym.to_json
p [1].to_json
p JSON.generate(Time.at(0).utc)
class Numeral < Numeric
  def to_s = "IV"
end
p JSON.generate([1..2, Numeral.new])
