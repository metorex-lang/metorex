class Patcher
  attr_reader :target

  def initialize = @target = :held

  def run
    first = "#{target}"
    target = :local
    [first, target]
  end

  def skipped
    total = 1 if false
    total
  end

  def in_a_loop
    2.times.map do
      seen = target
      target = :local
      seen
    end
  end
end

patcher = Patcher.new
p patcher.run
p patcher.skipped
p patcher.in_a_loop
