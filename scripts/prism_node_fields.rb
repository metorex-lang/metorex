# Writes src/vm/native_functions/prism_node_fields.rs, the table the native
# Prism loader reads nodes by, from the load_node that Prism generates in
# prism/serialize.rb. Run it after updating the vendored Prism:
#
#   ruby scripts/prism_node_fields.rb

ROOT = File.expand_path("..", __dir__)
SERIALIZE = File.join(ROOT, "src/vm/stdlib/prism/prism/serialize.rb")
OUTPUT = File.join(ROOT, "src/vm/native_functions/prism_node_fields.rs")

FIELDS = {
  "load_node(constant_pool, encoding, freeze)" => "Node",
  "load_optional_node(constant_pool, encoding, freeze)" => "OptionalNode",
  "Array.new(load_varuint) { load_node(constant_pool, encoding, freeze) }.tap { |nodes| nodes.freeze if freeze }" => "NodeList",
  "load_constant(constant_pool, encoding)" => "Constant",
  "load_optional_constant(constant_pool, encoding)" => "OptionalConstant",
  "Array.new(load_varuint) { load_constant(constant_pool, encoding) }.tap { |constants| constants.freeze if freeze }" => "ConstantList",
  "load_location(freeze)" => "Location",
  "load_optional_location(freeze)" => "OptionalLocation",
  "load_string(encoding)" => "String",
  "load_integer" => "Integer",
  "load_double" => "Double",
  "load_varuint" => "Varuint",
  "io.getbyte" => "Byte"
}.freeze

# The arguments of a call written out on one line, split at the commas that
# separate them.
def arguments_of(written)
  depth = 0
  current = +""
  listed = []
  written.each_char do |character|
    depth += 1 if "({".include?(character)
    depth -= 1 if ")}".include?(character)
    if character == "," && depth.zero?
      listed << current.strip
      current = +""
    else
      current << character
    end
  end
  listed << current.strip unless current.strip.empty?
  listed
end

text = File.read(SERIALIZE)
body = text[/if RUBY_ENGINE == "ruby"\n\s+def load_node.*?\n\s+end\n\n\s+value\.freeze/m] or abort "no load_node in #{SERIALIZE}"
major = text[/MAJOR_VERSION = (\d+)/, 1]
minor = text[/MINOR_VERSION = (\d+)/, 1]
patch = text[/PATCH_VERSION = (\d+)/, 1]

nodes = []
body.scan(/when (\d+) then\n(.*?)(?=\n\s+when \d+ then|\n\s+end\n)/m) do |number, code|
  lines = code.strip.lines.map(&:strip)
  skips_length = lines.first == "load_uint32"
  lines.shift if skips_length
  abort "unexpected load for node #{number}: #{code}" unless lines.size == 1
  name, written = lines.first.match(/\A(\w+)\.new\((.*)\)\z/).captures
  listed = arguments_of(written)
  head = listed.shift(4)
  abort "unexpected arguments for #{name}" unless head == %w[source node_id location load_varuint]
  fields = listed.map { |argument| FIELDS.fetch(argument) { abort "unknown field #{argument} of #{name}" } }
  abort "node #{number} out of order" unless number.to_i == nodes.size + 1
  nodes << [name, skips_length, fields]
end

File.write(OUTPUT, <<~RUST)
  // Written by scripts/prism_node_fields.rb from Prism #{major}.#{minor}.#{patch}'s
  // prism/serialize.rb. Run the script again rather than editing this file.

  use super::prism_loading::Field::{self, *};

  /// The serialization version this table reads.
  pub(crate) const SERIALIZATION_VERSION: [u8; 3] = [#{major}, #{minor}, #{patch}];

  /// Each node type, numbered from 1: its class under Prism, whether a
  /// four-byte length comes before its flags, and the fields after them.
  pub(crate) const NODE_TYPES: [(&str, bool, &[Field]); #{nodes.size}] = [
  #{nodes.map { |name, skips, fields| "    (#{name.inspect}, #{skips}, &[#{fields.join(", ")}])," }.join("\n")}
  ];
RUST
puts "wrote #{nodes.size} node types to #{OUTPUT}"
