def show(items)
  a = 1; b = 2; c = a + b
  items.each { |i| x = i; y = x }
  c
end
show([1, 2])
2.times { p1 = 1; p2 = 2 }
if true then q0 = 1; r0 = 2 end
i = 0; while i < 3; i += 1; end
j = 0; until j > 2 do j += 1 end
case i when 3 then a = 1; b = 2 else c = 3 end
begin; d = 1; raise "x"; rescue; e = 2; end
unless false then f = 1; g = 2 end
h = 1 if true; k = 2
def one_line(x) = (y = x; y + 1)
one_line(1); one_line(2)
[1, 2].each do |n| m = n; o = m end
for q in [1, 2] do r = q; s2 = r end
x = 1
y = x; z = y
case x
when 1
  w1 = 1; w2 = 2
end
