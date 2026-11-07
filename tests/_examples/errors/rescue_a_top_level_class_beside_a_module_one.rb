module Outer
  LoadError = Class.new(StandardError)
  def self.one
    require "foo/nothing"
  rescue ::LoadError
    :top
  end
  def self.two
    raise ArgumentError, "x"
  rescue ::ArgumentError
    :argument
  end
  def self.three
    require "foo/nothing"
  rescue LoadError
    :lexical
  end
  def self.four
    begin
      require "foo/nothing"
    rescue ::LoadError, ::ArgumentError
      :pair
    end
  end
end
%i[one two three four].each { |name| p(begin; Outer.send(name); rescue Exception => e; [:escaped, e.class]; end) }
