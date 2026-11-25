class UnknownRecord < NameError
  def initialize(record_name)
    super("no record named `#{record_name}`", record_name)
  end
end

error = UnknownRecord.new("Invoice")
puts(error.message)
p error.name

class MissingMethod < NoMethodError
  def initialize(method_name)
    super("no method #{method_name}", method_name, [1, 2])
  end
end

missing = MissingMethod.new(:total)
puts(missing.message)
p missing.name
p missing.args

class LookupFailed < KeyError
  def initialize(key)
    super("missing #{key}", receiver: {}, key: key)
  end
end

failed = LookupFailed.new(:limit)
puts(failed.message)
p failed.key
p failed.receiver

begin
  raise UnknownRecord, "Order"
rescue NameError => raised
  puts(raised.message)
  p raised.name
end
