class Registry
  class << self
    private

    def hidden(name)
      name
    end
  end
end

class Settings
  def alias_name(name)
    Registry.hidden(name)
  end

  def missing_name
    Registry.not_defined
  end
end
