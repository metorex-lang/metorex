class Painted < String
end

painted = Painted.new("ᗧ.")
quoted = Regexp.quote(painted)
p quoted
p quoted.encoding
p quoted.class
p Regexp.escape(Painted.new("a+b")).encoding
p("xᗧ.y" =~ /#{Regexp.quote(painted)}/)
