class Probe
  def plain
    1
  end

  def endless? = true || false

  # a comment after it
  def next_one = 2
  def with_args(value) = value
end

p Probe.instance_method(:plain).source_location[1]
p Probe.instance_method(:endless?).source_location[1]
p Probe.instance_method(:next_one).source_location[1]
p Probe.instance_method(:with_args).source_location[1]
