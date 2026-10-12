Reading = Data.define :meter, :value

# Keywords may name a member by String as well as by Symbol.
reading = Reading.new "meter" => "M-12", value: 40
p reading

# An object answering `to_str` names the member it spells.
key = Object.new
def key.to_str = "value"
reading = Reading.new meter: "M-12", key => 41
p reading

# Anything else is refused.
begin
  Reading.new meter: "M-12", 3 => 41
rescue TypeError => problem
  p problem.message
end

# An unknown key is reported the way it was written.
begin
  Reading.new meter: "M-12", value: 1, "unit" => "kWh"
rescue ArgumentError => problem
  p problem.message
end

# `deconstruct_keys` reads String keys too.
reading = Reading.new meter: "M-12", value: 42
p reading.deconstruct_keys ["meter"]

# A block handed to `new` reaches an `initialize` that asks for it.
Labeled = Data.define :name do
  def initialize(name:, &block)
    super name: block ? block.call(name) : name
  end
end
labeled = Labeled.new(name: "pump") { |held| held.upcase }
p labeled

# A `**rest` parameter hands back keys that are not Symbols.
def gather(**rest) = rest
gathered = gather **{ "zone" => 4, 7 => :seven }
p gathered
