versions = ((0.0r..1.0r) % 0.1r).to_a
p versions.size, versions.first(3), versions.last
p versions.select { _1 <= "0.5.12".to_r }
p "0.5.12".to_r, "0.4.3".to_r
p((0.0r..1.0r) % 0.1r)
p((0r..1r).step(1/4r).to_a)
p versions.each_cons(2).first(2)
p((1r..) % 2)
p((1.0...3.5).step(0.5))
p 1r.step(3, 1/2r)
p 1.step(10, 3r)
p (1r..2r).to_s, (1..).to_s, (1r..).to_s, (..2r).to_s, "#{1r..2r}", (1.5..2).to_s
