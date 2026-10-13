module Settings
  def level=(value)
    @level = value
  end
end

class Widget
  def size=(value)
    @size = value
  end
end

[-> { Settings.level = 1 }, -> { Widget.size = 2 }, -> { Widget.color = 3 }].each do |assignment|
  assignment.call
rescue NoMethodError => error
  puts error.message
end

p Widget.class_variables
