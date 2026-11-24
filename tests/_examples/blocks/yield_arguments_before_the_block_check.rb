def checkout
  raise ArgumentError, "no connection"
end

def with_connection
  yield checkout
end

begin
  with_connection
rescue ArgumentError, LocalJumpError => error
  p error.class
end

def bare_yield
  yield
end

begin
  bare_yield
rescue LocalJumpError => error
  p error.message
end
