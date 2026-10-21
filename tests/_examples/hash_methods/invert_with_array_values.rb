# Thor's `map` takes an inverted Hash whose keys are Arrays of aliases, and
# reads each alias out of them, as Bundler's `--version` is mapped.
aliases = { "version" => ["-v", "--version"], "exec" => "e" }
inverted = aliases.select { |name, _| name == "version" }.invert
p inverted
mapped = {}
inverted.each do |key, value|
  if key.respond_to?(:each)
    key.each { |one| mapped[one] = value }
  else
    mapped[key] = value
  end
end
p mapped
def map(mappings = nil, **keywords)
  p [mappings, keywords]
end
map inverted
map "-v" => "version"
