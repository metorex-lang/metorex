module Removing
  def self.remove(list, noop: nil)
    puts "remove #{list} noop=#{noop.inspect}"
  end

  def remove(list, noop: nil)
    puts "remove #{list} noop=#{noop.inspect}"
  end
  module_function :remove
end

module Quiet
  include Removing

  class << self
    attr_accessor :flag
  end
  Quiet.flag = false

  module_eval(<<-EOS, __FILE__, __LINE__ + 1)
    def remove(*args, **options, &block)
      super(*args,
          noop: Quiet.flag,
          **options, &block)
    end
  EOS

  def quietly
    old = Quiet.flag
    Quiet.flag = true
    yield
  ensure
    Quiet.flag = old
  end
end

class Task
  include Quiet

  def run
    quietly { remove "a" }
    remove "b"
    remove "c", noop: true
  end
end

Task.new.run
