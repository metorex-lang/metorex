p Regexp.new('^\P$').match?("P"), Regexp.new('^\P$').match?("\\P")
p "abc".scan(Regexp.new('[\u{0061 0062}]'))
p Regexp.new('[a]\g').match?("ag"), Regexp.new('\g').match?("g"), Regexp.new('\g').source
p Regexp.new('\p').match?("p") rescue p $!
p Regexp.new('a\gb').match?("agb")
