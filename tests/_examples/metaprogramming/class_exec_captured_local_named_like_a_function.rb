class Group
  def self.describe(&block)
    Class.new(self).class_exec(&block)
  end

  def self.it(&block)
    (@examples ||= []) << block
    new.instance_exec(&block)
  end
end

[:be_a, :be_an].each do |method|
  Group.describe do
    it { p send(method, 1) rescue p $! }
    it { p method }
  end
end
