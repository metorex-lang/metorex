class Lazily < Module
  def initialize(attributes)
    @attributes = attributes.map(&:to_s)
  end

  def ==(other)
    self.class == other.class && attributes == other.attributes
  end

  protected

  attr_reader :attributes
end

class Record
end

def setup(klass)
  defined = Lazily.new([:terms])
  klass.include(defined) unless klass.included_modules.include?(defined)
end

before = Record.ancestors.count
setup(Record)
setup(Record)
p Record.ancestors.count - before
p [Lazily.new([:a])].include?(Lazily.new([:a]))
p Lazily.new([:a]) == Lazily.new([:a])
p [Lazily.new([:a])].index(Lazily.new([:a]))
