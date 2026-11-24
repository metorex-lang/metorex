extension = Module.new do
  def const_missing(name)
    name == :Error ? ArgumentError : super
  end
end
legacy = Module.new
legacy.singleton_class.prepend(extension)
p legacy::Error
begin
  raise ArgumentError
rescue legacy::Error
  p :rescued
end
plain = Module.new
plain.extend(extension)
p plain::Error
begin
  plain::Other
rescue NameError => error
  p error.message.sub(/0x\h+/, "0x")
end
begin
  Module.new::Other
rescue NameError => error
  p error.message.sub(/0x\h+/, "0x")
end
m = Module.new
m.extend(Module.new { def const_missing(n) = super })
begin
  m::Other
rescue NameError => error
  p error.message.sub(/0x\h+/, "0x")
end
