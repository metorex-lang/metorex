p "(a(b)c)"[/\((?:[^()]|\g<0>)*\)/]
p "[#<Class:#<Object:0x1>>, :m]".gsub(/#<(?:[^<>]|\g<0>)+>/) { |segment| %("#{segment}") }
