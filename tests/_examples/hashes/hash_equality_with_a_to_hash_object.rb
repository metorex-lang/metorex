class Properties
  attr_reader :data

  def initialize(data)
    @data = data
  end

  def ==(other)
    data == other
  end

  def respond_to_missing?(name, include_private = false)
    data.respond_to?(name, include_private) || super
  end

  def method_missing(name, *arguments, &block)
    if data.respond_to?(name)
      data.public_send(name, *arguments, &block)
    else
      super
    end
  end
end

one = Properties.new({ key: "1" })
two = Properties.new({ key: "1" })
p one == two
p({ key: "1" } == two)
p({ key: "2" } == two)
p [1] == Struct.new(:to_ary).new([1])
