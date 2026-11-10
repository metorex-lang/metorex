# An operator after the end of an if, unless or case applies to the value
# the construct answers, as nokogiri's `end =~ /of-type/` does.
def f(c)
  if c
    "first-of-type"
  else
    "last"
  end =~ /of-type/
end
p f(true), f(false)
def g(c)
  if c then 1 else 2 end + 10
end
p g(true)
def k(c)
  case c when 1 then "a" else "b" end * 2
end
p k(1)
def m(c)
  unless c then 5 else 6 end - 1
end
p m(false)
