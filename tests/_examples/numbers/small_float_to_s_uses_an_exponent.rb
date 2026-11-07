p (16 * 200 / 1762500000.123).to_s
p 2.35e-06.to_s, 0.0001.to_s, 0.00001.to_s, 1.0e16.to_s, 1.0e15.to_s
p (16 * 200 / 1762500000.123).to_s.slice(-3..-1).to_i
p Thread.current.object_id < 100
