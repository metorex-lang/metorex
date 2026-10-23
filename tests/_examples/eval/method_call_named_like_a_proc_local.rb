def message(msg = nil, &default)
  return msg if Proc === msg
  proc { "#{msg}|#{default.call}" }
end
message = proc { "blah1" }
p message(message) { "blah2" }.call
message = message { "blah1" }
p message.class
p message(message) { "blah2" }.call
obj = Object.new
def obj.to_str; "blah" end
p(/blah/ =~ obj)
p Regexp.last_match
