module Holder
  BAR = "foo bar"
end

class Forwarding < Module
  def initialize(target)
    @target = target
  end

  instance_methods.each { |name| undef_method name unless /^__|^object_id$/.match?(name) }

  def respond_to?(*args)
    @target.respond_to?(*args)
  end

  def extended(base)
    puts "extended"
    base.extend(@target)
  end

  private

  def const_missing(name)
    puts "const_missing #{name}"
    @target.const_get(name)
  end

  def method_missing(name, *args, &block)
    puts "forwarding #{name}"
    @target.__send__(name, *args, &block)
  end
end

text_proxy = Forwarding.new("foo bar")
p "foo bar" == text_proxy
p text_proxy.respond_to?(:to_str)
p text_proxy == "foo bar"
p "foo bar".==(text_proxy)
