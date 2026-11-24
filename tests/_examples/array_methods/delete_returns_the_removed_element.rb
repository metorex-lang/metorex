Record = Struct.new(:id, :label) do
  def ==(other)
    other.is_a?(Record) && id == other.id
  end
end

in_memory = Record.new(1, "in memory")
loaded = Record.new(1, "loaded")
memory = [in_memory, Record.new(2, "other")]
p memory.delete(loaded).label
p memory.map(&:label)
p memory.delete(Record.new(9, "absent"))
p memory.delete(Record.new(9, "absent")) { :not_found }
