# A module prepended to a core class's singleton class reaches the class's
# own methods through super, as one defined with def does.
module CountedLookups
  define_method(:[]) do |*patterns|
    "counted #{super(*patterns).class}"
  end

  def exist?(path)
    "checked #{super}"
  end
end

Dir.singleton_class.prepend(CountedLookups)
File.singleton_class.prepend(CountedLookups)
p Dir["*.nothing-matches"]
p File.exist?("/")
