require "date"

day = Date.new(2004, 1, 2)
p Date.private_instance_methods(false).grep(/\A(hour|min|minute|sec|second)\z/).sort
p day.send(:hour), day.send(:min), day.send(:second)
begin
  day.hour
rescue NoMethodError => error
  p error.message
end
p DateTime.new(2004, 1, 2, 3, 4, 5).hour
p DateTime.public_method_defined?(:minute)
