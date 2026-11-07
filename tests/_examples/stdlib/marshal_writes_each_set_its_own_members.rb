require "set"

class Kind
  def initialize(name)
    @name = name
    @extensions = Set[]
  end
  attr_reader :name
end

class Shelf
  def initialize = @table = {}
  def add(key, value) = (@table[key] ||= Set.new) << value
  def marshal_dump = {}.merge(@table)
  def marshal_load(table) = @table = table
  def [](key) = @table[key]
end

shelf = Shelf.new
names = (1..40).map { |count| "type/#{count}" }
names.each { |name| shelf.add(name, Kind.new(name)) }
copied = Marshal.load(Marshal.dump(shelf))
p names.all? { |name| copied[name].first&.name == name }
