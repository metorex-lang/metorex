# A `super` that finds no method further up calls method_missing with the
# method's name, as a call that finds no method does.
class Mailer
  def self.method_missing(name, *args)
    name == :welcome ? "welcome via method_missing #{args.inspect}" : super
  end

  def self.respond_to_missing?(name, include_private = false)
    name == :welcome || super
  end
end

Mailer.singleton_class.define_method(:welcome) do |*args|
  super(*args)
end
puts Mailer.welcome(1, 2)

class Greeter
  def method_missing(name, *args)
    "#{name} reached method_missing"
  end

  def respond_to_missing?(*) = true

  def hello
    super
  end
end
puts Greeter.new.hello

class Plain
  def greet
    super
  end
end
begin
  Plain.new.greet
rescue NoMethodError => error
  puts error.class
  puts error.message
  puts error.name.inspect
end

class Report
  def self.build
    super
  end
end
begin
  Report.build
rescue NoMethodError => error
  puts error.message
  puts error.receiver
end
