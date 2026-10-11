def entries_from(&block)
  seen = []
  block.call { |key, value| seen << [key, value] }
  seen
end

class RateTable < Hash
  def entries_seen = entries_from(&method(:each))
end

table = RateTable.new
table["shipping"] = 4
p(table.entries_seen)
p(entries_from(&{ "tariff" => 9 }.method(:each)))

def forward(amount, &block)
  block.call(amount)
end
p(method(:forward).to_proc.call(3) { |amount| amount * 10 })

def pair_of(key, value) = [key, value]
begin
  { "tariff" => 9 }.each(&method(:pair_of))
rescue ArgumentError => error
  p(error.message)
end
takes_block = ->(amount, &block) { block.call(amount) }
begin
  takes_block.call
rescue ArgumentError => error
  p(error.message)
end
