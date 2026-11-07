p((Encoding.find(nil) rescue [$!.class, $!.message]))
p((Encoding.find(5) rescue [$!.class, $!.message]))
p((Encoding.find(:utf_8) rescue [$!.class, $!.message]))
p Encoding.find("utf-8")
