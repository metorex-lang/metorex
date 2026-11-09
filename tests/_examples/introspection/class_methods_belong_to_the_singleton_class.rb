class Reading
  def self.latest; end
end
latest = Reading.method(:latest)
p latest.owner
p latest.owner.instance_method(:latest).owner
p latest.unbind.bind(Reading).inspect.start_with?("#<Method: Reading.latest()")
anonymous = Class.new { def self.latest; end }
p anonymous.method(:latest).owner == anonymous.singleton_class
p Class.method(:new).owner
