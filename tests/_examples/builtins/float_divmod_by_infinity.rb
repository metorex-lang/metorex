[-4.2, 0.0, 4.2, -0.0].each do |x|
  p x.divmod(Float::INFINITY), x.divmod(-Float::INFINITY)
end
p 7.5.divmod(2), -7.5.divmod(2), 7.5.divmod(-2.0), 1.0.divmod(0.3)
p 6.0.divmod(2**70)
begin
  Float::INFINITY.divmod(2)
rescue FloatDomainError => error
  p error.message
end
begin
  1.0.divmod(Float::NAN)
rescue FloatDomainError => error
  p error.message
end
