class Forwarding < Module
  def initialize(target)
    @target = target
  end

  instance_methods.each { |name| undef_method name unless /^__|^object_id$/.match?(name) }

  def method_missing(name, *args, &block)
    puts "forwarding #{name}"
    @target.__send__(name, *args, &block)
  end
end
proxy = Forwarding.new(Array)
made = proxy.new(2, 0)
p made
begin
  Module.new.new
rescue NoMethodError => error
  p error.message.sub(/0x\h+/, "0x")
end
module Named; end
begin
  Named.new
rescue NoMethodError => error
  p error.message.sub(/0x\h+/, "0x")
end
