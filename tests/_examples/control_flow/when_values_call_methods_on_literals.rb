def kind(value)
  case value
  when 0.class then :integer
  when "text".class then :string
  when 1.5.class then :float
  else :other
  end
end

p kind(7)
p kind("seven")
p kind(7.0)
p kind(nil)
