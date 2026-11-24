def show(&block)
  match = /(b)(c)/.match("abc")
  block.binding.eval("$~ = nil")
  p block.call
  block.binding.eval("proc { |m| $~ = m }").call(match)
  p block.call
end
show { $~ && $2 }
def directly(&block)
  "xy".sub(/(x)(y)/) { block.binding.eval("proc { |m| $~ = m }").call($~); block.call }
end
p directly { $2 + $1 }
