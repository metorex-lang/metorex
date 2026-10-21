written = %w[12abc 1.5e2 -0.25.1 1_000.5 x 1.2/3 1/3.5 .5 5. 1/1e1 3/x 1e 1e+ 2e-1 1__0 _1 1_ -.5 +3 1/-2 1/+2 1.5/2.5 0x10 1/2/3 1e2/1e1 1.5.5 1/2e 0.5.12]
written.each { |text| puts "#{text} #{text.to_r.inspect}" }
p " \t1/3".to_r, "1 /3".to_r
begin
  "3/0.0".to_r
rescue ZeroDivisionError => error
  p error.message
end
