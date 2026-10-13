def load_once(attempts)
  attempts << :tried
  raise ArgumentError, "first try" if attempts.size < 2

  attempts
rescue ArgumentError
  retry
end

p load_once([])

def load_with_condition(attempts)
  attempts << :tried
  raise ArgumentError, "first try" if attempts.size < 3

  attempts
rescue ArgumentError
  attempts.size < 3 ? retry : raise
end

p load_with_condition([])

def give_up(attempts)
  attempts << :tried
  raise ArgumentError, "always"
rescue ArgumentError
  attempts.size < 2 ? retry : raise
end

begin
  give_up([])
rescue ArgumentError => error
  p error.message
end

def retry_with_if(attempts)
  attempts << :tried
  raise ArgumentError, "first try" if attempts.size < 2

  attempts
rescue ArgumentError
  retry if attempts.size < 2
end

p retry_with_if([])
