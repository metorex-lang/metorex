module Coded
  attr_reader :code

  def initialize(message = nil, code = nil)
    super(message)
    @code = code
  end
end

module Retrying
  def set_attempt(attempt)
    @attempt = attempt
  end
end

module Final
  def set_attempt(_attempt)
  end
end

class BaseFailure < StandardError
  include Retrying
end

class CommandFailure < BaseFailure
  include Coded
  include Final
end

WrongType = Class.new(CommandFailure)

[CommandFailure, WrongType].each do |klass|
  error = klass.new("WRONGTYPE bad key", "WRONGTYPE")
  p [error.class, error.message, error.code, error.set_attempt(1)]
end
