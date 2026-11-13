module Runner
  def self.load_files(names)
    names.each { |name| require "./#{name}" }
  end

  def self.run(object, names)
    names.each { |name| object.send(name) }
  end
end
