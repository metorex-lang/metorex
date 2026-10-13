def reraise_plain
  raise "original"
rescue => error
  raise ArgumentError.new("replacement")
end

def reraise_without_cause
  raise "original"
rescue => error
  replacement = NameError.new("replacement", :name)
  replacement.set_backtrace(error.backtrace)
  raise replacement, cause: nil
end

begin
  reraise_plain
rescue => error
  p error.cause&.message
end

begin
  reraise_without_cause
rescue => error
  p error.cause
end
