class Settings
  def initialize
    @included = []
  end

  def include(mod, *filters)
    @included << [mod, filters]
  end

  def expose
    include Comparable
    include Enumerable, :tagged
    @included
  end
end

p Settings.new.expose
p Settings.include?(Comparable)
