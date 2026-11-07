class Ledger
  def inspect = "Ledger!"
end

class Tags < Set; end
p Set[Ledger.new, "x", :y, 1.5, nil, [1]]
p Tags[1, 2], Tags.new([3]).inspect, Tags[].to_s
p Set[Tags[1]], [Tags[2]]
looped = Tags[1]
looped << looped
p looped
plain = Set[1]
plain << plain
p plain
