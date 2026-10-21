module Kit
  module Host
    def [](name)
      require File.join(__dir__, name.to_s)
      const_get(name.to_s.capitalize)
    end
  end
end
