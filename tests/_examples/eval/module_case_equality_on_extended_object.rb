error = Exception.new("Error").extend(Math)
p Math === error
p error.is_a?(Math)
p error.singleton_class.include?(Math)
plain = Object.new.extend(Comparable)
p Comparable === plain
begin
  raise error
rescue Math => caught
  p caught.message
end
