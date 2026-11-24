class Wrapping
  def initialize(text)
    @text = text
  end

  def method_missing(name, *args, &block)
    result = @text.__send__(name, *args, &block)
    result.is_a?(String) ? Wrapping.new(result) : result
  end

  def respond_to_missing?(name, include_private = false)
    @text.respond_to?(name, include_private)
  end

  def to_s
    @text
  end
end
held = Wrapping.new("ab")
p (held + "cd").to_s
p (+held).to_s
p (-held).to_s
p held.method(:+).call("x").to_s
p (held * 2).to_s
p held == "ab"
