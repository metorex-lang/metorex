# The fib benchmark's method, called 21,891 times by fib(20).
def fib(n)
  if n < 2
    return n
  end

  return fib(n-1) + fib(n-2)
end

p fib(20)
