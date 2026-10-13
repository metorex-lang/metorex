class Fake < Exception
  attr_reader :backtrace
  if true
    attr_accessor :cause
  end
end
p Fake.instance_methods(false).sort
