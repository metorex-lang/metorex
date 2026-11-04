def Warning.warn(message)
  print message.sub(%r{\A.*/}, "")
end

def inner
  warn "from inner", uplevel: 1
end

def outer
  inner
end

outer
[1].each { |item| warn "in a block", uplevel: 1 }
warn "past the top", uplevel: 10
warn "here", uplevel: 0
