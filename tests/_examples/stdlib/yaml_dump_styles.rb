require "yaml"

Reading = Struct.new(:sensor, :value)
document = {
  "plain" => "maintenance window",
  "quoted" => "02:00",
  "boolean_word" => "yes",
  "empty" => "",
  "multiline" => "first line\nsecond line\n",
  "trailing_space" => "ends with a space ",
  "unicode" => "café ☃",
  "control" => "tab\there",
  "symbol" => :ready,
  "numbers" => [1, 2.5, -3, 10**20],
  "nothing" => nil,
  "flags" => [true, false],
  "nested" => { "empty_list" => [], "empty_map" => {}, "list" => %w[a b] },
  "struct" => Reading.new("north", 21.5),
  "range" => (1..3),
  "long" => "word " * 30,
}
puts YAML.dump(document)
puts [1, [2, [3]]].to_yaml
puts YAML.dump_stream({ "a" => 1 }, "two", [3])
shared = %w[x y]
puts YAML.dump("first" => shared, "second" => shared)
p YAML.load(YAML.dump(document.reject { |key, _| %w[struct range symbol].include?(key) }))
