# `min` and `max` take a count and answer that many values, in order.
p [7, 2, 9, 4].min(2)
p [7, 2, 9, 4].max(2)
p [7, 2, 9, 4].max(2) { |left, right| right <=> left }
p [].min(3)
p [3].min(nil)
begin
  [1, 2].min(-1)
rescue ArgumentError => problem
  p problem.message
end

# A range of Integers counts from the end asked for without walking, so a
# range too large to walk still answers.
p (1..10).max(3)
p (0...2**64).max(2)
p (1..).min(2)
p (..5).max(2)
p ("f".."l").max(2)
p (100..10).min(2)
p (1..3.5).max(2)
p (1..3.5).min(2)
[-> { (1.0..).min(2) }, -> { (..1.0).max(2) }, -> { (1.0..3).max(2) }, -> { (0..2).max(-1) }].each do |call|
  begin
    call.call
  rescue TypeError, ArgumentError => problem
    p problem.message
  end
end

# `find` takes the value to answer when nothing matches, and hands out an
# Enumerator when no block is given.
p [1, 2, 3].find(-> { :none }) { |held| held > 5 }
p [1, 2, 3].find.class
p [1, 2, 3].detect(-> { :none }).each { |held| held > 1 }
seen = []
shrinking = [4, 2, 1]
shrinking.find { |held| seen << held; shrinking.clear; false }
p seen
