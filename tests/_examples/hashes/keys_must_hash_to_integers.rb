require "set"
broken = Module.new { def self.hash; end }
record = Object.new
def record.hash = nil
[
  -> { Set.new([Kernel]).include?(broken) },
  -> { { Kernel => 1 }.key?(broken) },
  -> { { record => 1 } }
].each do |attempt|
  attempt.call
rescue TypeError => error
  p error.message
end
spread = Object.new
def spread.hash = 2**70
def spread.eql?(other) = equal?(other)
p({ spread => 1 }[spread])
