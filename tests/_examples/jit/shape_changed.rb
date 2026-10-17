class Reading
  def initialize(value)
    @value = value
  end

  def value = @value

  def add_unit
    @unit = "ms"
  end
end

readings = Array.new(4) { |index| Reading.new(index * 10) }
readings[2].remove_instance_variable(:@value)
readings[2].instance_variable_set(:@value, 25)
readings.each { |reading| puts reading.value }
readings[1].add_unit
readings.each { |reading| puts reading.value }
