# A block written in a loop and handed out again on each pass reads the
# locals as they stand on that pass, and a block something kept is never
# handed out again.
def take(&block) = block
def run_it = yield

kept = []
3.times do |pass|
  kept << take { pass }
  p run_it { pass * 10 }
end
p kept.map(&:call)
p kept.uniq(&:object_id).size

total = 0
4.times { |step| run_it { total += step } }
p total

procs = []
2.times do
  block = take { :same }
  procs << block
  procs << take { :same }
end
p procs.map(&:object_id).uniq.size

require "weakref"
def held_by_a_block
  captured = Object.new
  run_it { captured }
  WeakRef.new(captured)
end
reference = held_by_a_block
GC.start
p reference.weakref_alive?
