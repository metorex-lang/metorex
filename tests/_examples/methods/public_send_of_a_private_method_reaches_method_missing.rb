# public_send cannot call a private method, so a receiver with method_missing
# is asked about the name, as a call written with a receiver asks it.
class Mailer
  def self.method_missing(name, *args)
    "#{name} answered by method_missing with #{args.inspect}"
  end

  def self.respond_to_missing?(*) = true
end

puts Mailer.public_send(:test, 1)
puts Mailer.public_send(:puts, "hello")
puts Mailer.send(:format, "%d items", 3)

class Plain
  private

  def secret = "hidden"
end
begin
  Plain.new.public_send(:secret)
rescue NoMethodError => error
  puts error.message
end
