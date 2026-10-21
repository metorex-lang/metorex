settings = Hash.new { |_hash, key| caller(0, 3).map { |line| line.sub(__FILE__, "FILE") } }

group = Class.new
lines = group.class_exec do
  settings[:example_group]
end
puts lines
puts "--"
puts(proc { settings.fetch(:missing) { caller(0, 2).map { |line| line.sub(__FILE__, "FILE") } } }.call)
