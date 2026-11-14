# An Array subclass that aliases one native method under the name of
# another, as Markaby's Stream gives `join` the name `to_s`, answers what
# the aliased method does.
class Stream < Array
  alias_method :to_s, :join
end
class Lines < Array
  alias inspect join
end
stream = Stream.new(["<p>", "text", "</p>"])
p stream.to_s
puts "#{stream}"
p stream.send(:to_s), stream.method(:to_s).call
p Lines.new(["a", "b"]).inspect
