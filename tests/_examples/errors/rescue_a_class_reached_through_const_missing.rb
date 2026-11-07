module Current
  class ParseError < StandardError; end
end

module Legacy
  def self.const_missing(name)
    Current.const_get(name)
  end
end

p Object.const_get("Legacy::ParseError")
begin
  raise Current::ParseError, "boom"
rescue Legacy::ParseError => error
  p [:caught, error.message]
end
begin
  raise ArgumentError, "other"
rescue Legacy::ParseError
  p :wrong
rescue ArgumentError => error
  p [:argument, error.message]
end
