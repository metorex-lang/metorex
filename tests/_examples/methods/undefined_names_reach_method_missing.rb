class Blank
  undef_method :<=>
  undef_method :==

  def method_missing(name, *arguments)
    [:missing, name, *arguments]
  end

  def respond_to_missing?(*)
    true
  end
end

p Blank.new <=> 1
p Blank.new == 2
p Blank.new.__send__(:<=>, 3)

class Wrapper
  def method_missing(name, *arguments)
    [:missing, name, *arguments]
  end
end
p Wrapper.new < 4

module AnyStandardError
  def self.===(exception)
    exception.is_a?(StandardError)
  end
end

begin
  raise ArgumentError, "custom match"
rescue AnyStandardError => error
  p error.message
end

p([Blank.new, Blank.new].sort) rescue p $!.class

class Quiet
  undef_method :inspect

  def method_missing(name, *arguments)
    [:missing, name]
  end
end
p Quiet.new.inspect

class Plain
  undef_method :hash
end
Plain.new.hash rescue p $!.message

class Ordered
  def <=>(other)
    0
  end

  def method_missing(name, *arguments)
    [:missing, name, *arguments]
  end
end
p Ordered.new < 5
