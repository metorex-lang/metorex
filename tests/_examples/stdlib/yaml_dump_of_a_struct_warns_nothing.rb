# Dumping a Struct to YAML prints no warning when warnings are on.
$VERBOSE = true
require "yaml"
Pair = Struct.new(:left, :right)
puts Pair.new(7, "something").to_yaml
puts YAML.dump(Pair.new(7, "something"))
