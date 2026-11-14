# A condition that assigns the value of an `||=` on an element, as
# pdf-reader writes it, and a chain of plain assignments in a condition.
class LoggedStreams < Hash
  def []=(key, value)
    puts "store #{key}"
    super
  end
end
streams = LoggedStreams.new
2.times do
  if stream = streams[:objects] ||= "object stream"
    p stream
  end
end
count = nil
while first = second = count ||= 0
  count += 1
  break if count > 2
end
p [first, second, count]
