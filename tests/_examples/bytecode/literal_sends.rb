# send, __send__, and public_send with a Symbol written as the name reach the
# method by that name, with its arguments and block, and a send the program
# wrote itself runs in their place.
class Greeter
  def greet(name, punctuation: "!") = "hello #{name}#{punctuation}"
  def each_word = %w[one two].each { |word| yield word }

  private

  def secret = :secret
end

greeter = Greeter.new
p greeter.send(:greet, "you")
p greeter.__send__(:greet, "you", punctuation: "?")
p greeter.public_send(:greet, "them")
words = []
greeter.send(:each_word) { |word| words << word }
p words
p greeter.send(:secret)
p(begin
  greeter.public_send(:secret)
rescue NoMethodError => error
  error.message
end)
p [1, 2, 3].send(:sum)
p([1, 2, 3].send(:find) { |number| break :broke if number == 2 })

class Mailer
  def send(message, *) = "mailed #{message}"
end
p Mailer.new.send(:greet, "you")

class Greeter
  def public_send(*) = :own
end
p greeter.public_send(:greet, "you")
