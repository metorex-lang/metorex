Reading = Data.define(:meter, :value)

# Keywords may name a member by String as well as by Symbol.
p Reading.new("meter" => "M-12", value: 40)

# An object answering `to_str` names the member it spells.
key = Object.new
def key.to_str = "value"
p Reading.new(meter: "M-12", key => 41)

# Anything else is refused.
begin
  Reading.new(meter: "M-12", 3 => 41)
rescue TypeError => problem
  p problem.message
end

# An unknown key is reported the way it was written.
begin
  Reading.new(meter: "M-12", value: 1, "unit" => "kWh")
rescue ArgumentError => problem
  p problem.message
end

# `deconstruct_keys` reads String keys too.
p Reading.new(meter: "M-12", value: 42).deconstruct_keys(["meter"])

# A block handed to `new` reaches an `initialize` that asks for it.
Labeled = Data.define(:name) do
  def initialize(name:, &block)
    super(name: block ? block.call(name) : name)
  end
end
p Labeled.new(name: "pump") { |held| held.upcase }

# A `**rest` parameter hands back keys that are not Symbols.
def gather(**rest) = rest
p gather(**{ "zone" => 4, 7 => :seven })
