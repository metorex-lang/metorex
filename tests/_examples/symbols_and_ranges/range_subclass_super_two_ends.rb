class LineRange < Range
  attr_reader :note
  def initialize(first, last, note)
    super(first, last)
    @note = note
  end
end
r = LineRange.new(1, 3, "x")
p r, r.exclude_end?, r.max, r.note
p LineRange.new(2, 2, "y") == (2..2)
p LineRange.new(4, 6, "z").end >= 5
