%w[USR1 USR2 INT TERM HUP QUIT PIPE CHLD ALRM WINCH TTIN].each do |name|
  first = Signal.trap(name, "IGNORE")
  second = Signal.trap(name, "SYSTEM_DEFAULT")
  third = Signal.trap(name, "DEFAULT")
  p [name, first, second, third]
end
