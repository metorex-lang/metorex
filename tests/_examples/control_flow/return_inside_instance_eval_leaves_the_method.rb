class Holder
  def initialize(ready)
    @ready = ready
  end

  def ready?
    @ready
  end
end

def all_ready?(holders)
  holders.all? do |holder|
    holder.instance_eval do
      return false unless ready?
    end
  end
  true
end

p all_ready?([Holder.new(false), Holder.new(true)])

def first_value
  Holder.new(true).instance_eval { return :from_eval }
  :after
end
p first_value
