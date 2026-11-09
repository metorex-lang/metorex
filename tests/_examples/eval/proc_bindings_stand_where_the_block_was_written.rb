b = proc { 1 }.binding
p File.basename(b.source_location[0]), b.source_location[1]
