class Counter
  attr_accessor :assertions

  def initialize
    @assertions = 3
  end
end

instance = Counter.new

def instance.assertions
  if @forward
    @forward.assertions
  else
    super
  end
end

def instance.assertions=(value)
  if @forward
    @forward.assertions = value
  else
    super
  end
end

p instance.assertions
instance.assertions = 5
p instance.assertions
copy = instance.dup
p copy.assertions
