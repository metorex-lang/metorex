module Reporter
  class << self
    def label
      "from singleton class body"
    end
  end
end

module Reporter
  def self.label
    "from def self"
  end
end
puts Reporter.label
p Reporter.method(:label).source_location[1]

class Counter
  def self.total
    "from def self"
  end

  class << self
    def total
      "from singleton class body"
    end
  end
end
puts Counter.total
