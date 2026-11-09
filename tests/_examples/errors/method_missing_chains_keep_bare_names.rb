module Matchers
  def method_missing(name, *args, &block)
    return :matched if name.to_s.start_with?("be_")
    super
  end
end
class Group
  include Matchers
  def method_missing(name, *args)
    super(name, *args)
  end
  def run
    acount[:balance]
  rescue NameError => e
    p [e.class, e.message]
  end
end
Group.new.run
