class Pattern < Regexp; end
pattern = Pattern.new("lib/")
p "lib/a.rb".match?(pattern), "src/a.rb".match?(pattern)
