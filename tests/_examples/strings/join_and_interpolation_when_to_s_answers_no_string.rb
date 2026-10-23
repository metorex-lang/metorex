k = Class.new { def self.to_s; nil; end }
p [k, "inner"].join("::").sub(/0x\h+/, "ID")
o = Object.new
def o.to_s; 5; end
p [o].join.sub(/0x\h+/, "ID")
p "#{o}".sub(/0x\h+/, "ID")
