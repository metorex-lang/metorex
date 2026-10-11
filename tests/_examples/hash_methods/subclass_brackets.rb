class StringKeyHash < Hash
  def [](key)
    super(key.to_s)
  end

  def []=(key, value)
    super(key.to_s, value)
  end
end

observers = StringKeyHash.new
observers[:rates] = [:reload]
observers[:tariffs] ||= []
observers[:tariffs] << :refresh
observers[:rates] += [:audit]
p(observers)
p(observers[:tariffs])
