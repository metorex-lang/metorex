require "set"

def ids_taken
  before = Object.new.object_id
  yield
  Object.new.object_id - before - 8
end

holder = Object.new
taken = {}
taken['holder.hash'] = ids_taken { holder.hash }
taken['holder.inspect'] = ids_taken { holder.inspect }
taken['holder.to_s'] = ids_taken { holder.to_s }
taken['[holder].include?(holder)'] = ids_taken { [holder].include?(holder) }
taken['{ holder => 1 }[holder]'] = ids_taken { { holder => 1 }[holder] }
taken['{}.compare_by_identity[holder] = 1'] = ids_taken { {}.compare_by_identity[holder] = 1 }
taken['Set[holder].include?(holder)'] = ids_taken { Set[holder].include?(holder) }
taken['holder.dup'] = ids_taken { holder.dup }
taken['Marshal.load(Marshal.dump([1, "a"]))'] = ids_taken { Marshal.load(Marshal.dump([1, "a"])) }
taken['holder.instance_variable_set(:@kept, 1)'] = ids_taken { holder.instance_variable_set(:@kept, 1) }
taken['holder.frozen?'] = ids_taken { holder.frozen? }
taken['holder == holder'] = ids_taken { holder == holder }
taken['holder.eql?(holder)'] = ids_taken { holder.eql?(holder) }
taken['holder.equal?(holder)'] = ids_taken { holder.equal?(holder) }
taken['holder.method(:to_s)'] = ids_taken { holder.method(:to_s) }
taken['"text".hash'] = ids_taken { "text".hash }
taken[':name.to_proc'] = ids_taken { :name.to_proc }
taken['Struct.new(:field).new(1).hash'] = ids_taken { Struct.new(:field).new(1).hash }
taken['[1, 2].hash'] = ids_taken { [1, 2].hash }
taken['{ key: 1 }.hash'] = ids_taken { { key: 1 }.hash }
taken['1.0.hash'] = ids_taken { 1.0.hash }
taken['(1..2).hash'] = ids_taken { (1..2).hash }
taken['Comparable.instance_method(:==)'] = ids_taken { Comparable.instance_method(:==) }
p taken.reject { |_, count| count.zero? }
