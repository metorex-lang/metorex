LIMIT = 3

def read_limits
  seen = []
  4.times do |pass|
    seen << LIMIT + pass
    Object.send(:remove_const, :LIMIT) if pass == 1
    Object.const_set(:LIMIT, 100) if pass == 1
  end
  seen
end

puts read_limits.inspect
