def wrap
  out = +"<form>"
  out << yield << "</form>"
  out
end

def halve
  yield >> 1
end

p wrap { "body" }
p halve { 10 }
