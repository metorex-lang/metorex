def run(code)
  eval code, binding, "named.rb", 10
rescue SyntaxError => error
  p [error.message, error.path, error.backtrace, error.backtrace_locations]
rescue LocalJumpError => error
  p error.message
end
run "\n\nclass Foo; yield; end"
run "module Foo; yield; end"
run "class << self; yield; end"
run "yield"
run "class Foo; def a; yield; end; end; :defined"
p run("def z; yield; end; z { 5 }")
begin
  eval "yield", binding, "top.rb", 1
rescue SyntaxError => error
  p [error.message, error.backtrace]
end
