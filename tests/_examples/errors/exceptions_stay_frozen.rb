error = StandardError.new("disk full")
p error.frozen?
error.freeze
p error.frozen?
p error.dup.frozen?
begin
  error.instance_variable_set(:@retries, 3)
rescue FrozenError => refused
  p refused.class
end
