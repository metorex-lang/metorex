class Kind
  attr_reader :name, :note

  def initialize(name, note)
    @name = name
    @note = note
  end

  def eql?(other) = other.is_a?(Kind) && name == other.name
  def ==(other) = eql?(other)
  def hash = name.hash
end

first = Kind.new("text/html", "first")
second = Kind.new("text/html", "second")
copied = Marshal.load(Marshal.dump([first, second, first]))
p copied.map(&:note)
p copied[0].equal?(copied[2]), copied[0].equal?(copied[1])

words = ["same", "same"]
again = Marshal.load(Marshal.dump(words))
p again[0].equal?(again[1])
