# Each read of Rex's clock moves it on by the time the running example
# names, so the ranking does not depend on how fast the machine is.
module SteppedClock
  STEPS = { "takes three seconds" => 3.0, "takes one second" => 1.0, "takes five seconds" => 5.0 }.freeze

  def self.now
    @now ||= 0.0
    example = Rex.current_example
    @now += example ? STEPS.fetch(example.description, 0.0) : 0.0
  end
end

Rex::Core.define_singleton_method(:monotonic_now) { SteppedClock.now }

describe "an import" do
  it "takes one second" do
  end

  it "takes three seconds" do
  end
end

describe "an export" do
  it "takes five seconds" do
  end
end
