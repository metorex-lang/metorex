# Hash#delete_if and its relatives remove each entry as the block answers,
# so a block that raises leaves the removals before it in place. Their
# enumerators call back through the receiver's own method.
%i[delete_if reject! keep_if select! filter!].each do |name|
  hash = { "keep" => 1, "drop" => 2, "last" => 3 }
  seen = 0
  begin
    hash.public_send(name) do |key, _|
      seen += 1
      raise "stop" if seen == 2
      %i[keep_if select! filter!].include?(name) ? key != "keep" : key == "keep"
    end
  rescue RuntimeError
  end
  p [name, hash.keys]
end

class CountingHash < Hash
  attr_reader :calls

  def delete_if
    @calls = (@calls || 0) + 1
    block_given? ? super : super
  end
end

counted = CountingHash.new
counted["a"] = 1
counted["b"] = 2
counted.delete_if.each { |key, _| key == "a" }
p [counted.keys, counted.calls]
