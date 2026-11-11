def skipped_or_called
  lambda = yield
  return :skipped unless lambda
  lambda.call
end
p skipped_or_called { false }
p skipped_or_called { -> { :called } }
def read_twice
  lambda = 3
  x = lambda
  p x
  y = lambda; p y
end
read_twice
square = lambda { |n| n * n }
p square.(4)
