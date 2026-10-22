@expanded_name = "x"
value = "1"
puts %Q^#@expanded_name="#{value}"^
puts %q^plain #@expanded_name^
puts %^#@expanded_name and #{value}^
puts %Q(#@expanded_name)
puts %Q|#@expanded_name|
puts "#@expanded_name"
