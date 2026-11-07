require "set"

Item = Struct.new(:name)
first = Item.new("html")
second = Item.new("htm")
held = { "text/html" => Set[first, second], "text/plain" => Set[] }
back = Marshal.load(Marshal.dump(held))
p back.keys, back["text/html"].class, back["text/html"].size, back["text/html"].map(&:name).sort
p back["text/plain"], Marshal.load(Marshal.dump(Set[1, 2, 3])) == Set[1, 2, 3]

class Box
  def initialize(table)
    @table = table
  end

  def marshal_dump
    {}.merge(@table)
  end

  def marshal_load(hash)
    @table = hash
  end

  def lookup(key)
    @table[key]
  end
end
shared = Item.new("shared")
boxes = [Box.new("a" => Set[shared]), Box.new("b" => Set[shared])]
loaded = Marshal.load(Marshal.dump(boxes))
p loaded.map { |box| box.lookup(box.equal?(loaded[0]) ? "a" : "b").first.name }
p loaded[0].lookup("a").first.equal?(loaded[1].lookup("b").first)
p Marshal.dump(Set[1]).bytesize == Marshal.dump(Set[1]).bytesize
