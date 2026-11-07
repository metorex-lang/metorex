require "set"

class Kind
  attr_reader :name

  def initialize(name, extensions)
    @name = name
    @extensions = Set[*extensions].freeze
  end

  def extensions = @extensions.to_a

  def add_extensions(*more)
    @extensions = Set[*(@extensions.to_a + more)].freeze
  end
end

class Shelf
  def initialize
    @container = Hash.new { |hash, key| hash[key] = Set.new }
  end

  def add(key, value)
    @container[key] << value
  end

  def [](key) = @container[key]

  def marshal_dump = {}.merge(@container)

  def marshal_load(hash)
    @container = hash
  end
end

html = Kind.new("text/html", %w[html htm])
by_type = Shelf.new
by_type.add("text/html", html)
by_extension = Shelf.new
by_extension.add("html", html)
html.add_extensions("hex")

registry = [by_type, by_extension]
copied = Marshal.load(Marshal.dump(registry))
p copied[0]["text/html"].first.extensions
p copied[0]["text/html"].first.equal?(copied[1]["html"].first)
