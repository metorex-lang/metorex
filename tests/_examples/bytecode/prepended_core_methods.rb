# A method prepended to a core class answers a direct call, a call through
# send, and a call through public_send.
module Sending
  def first = :sent
end
Array.prepend(Sending)

p [1, 2].first
p [1, 2].send(:first)
p [1, 2].public_send(:first)
p [1, 2].__send__(:first)

class Hash
  def fetch(key) = :fetched
end
p({a: 1}.fetch(:a))
p({a: 1}.send(:fetch, :a))
