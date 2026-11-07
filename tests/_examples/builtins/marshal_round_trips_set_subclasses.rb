class Tags < Set; end
t = Tags[3, 4]
p t.class, t.to_a, t.include?(4)
p Marshal.dump(t)
back = Marshal.load(Marshal.dump(t))
p back.class, back.to_a
s = Set[1]
s.instance_variable_set(:@note, "kept")
r = Marshal.load(Marshal.dump(s))
p Marshal.dump(s), r.instance_variable_get(:@note), r.to_a
i = Set[1].compare_by_identity
p Marshal.load(Marshal.dump(i)).compare_by_identity?
