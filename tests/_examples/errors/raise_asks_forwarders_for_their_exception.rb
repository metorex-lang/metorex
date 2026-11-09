class Forwarder < BasicObject
  def initialize(error) = @error = error

  def method_missing(name, *arguments, &block)
    @error.public_send(name, *arguments, &block)
  end

  def respond_to_missing?(name, include_all = false) = @error.respond_to?(name, include_all)
end

begin
  raise Forwarder.new(NameError.new("missing record"))
rescue NameError => error
  p error
end

begin
  raise 5
rescue TypeError => error
  p error
end
