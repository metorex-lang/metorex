Wrapper = Struct.new(:underlying) do
  class << self
    remove_method :[]

    def [](value)
      [:own, value]
    end
  end
end
p Wrapper[1]
p Wrapper.new(2).underlying
