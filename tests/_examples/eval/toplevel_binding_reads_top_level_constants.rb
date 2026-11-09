class Version
  NUMBER = "1.5.0"
end

module Library
  class Version
  end

  def self.evaluate(code)
    eval(code, TOPLEVEL_BINDING.dup, "library.gemspec")
  end
end

p Library.evaluate("Version::NUMBER")
p Library.evaluate("Module.nesting")
