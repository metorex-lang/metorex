code = "def show\n  @name = 1;@output_buffer.append=( foo( );\n@output_buffer\nend\n"

begin
  Class.new.module_eval(code, "/views/test/syntax_error.html.erb", -1)
rescue SyntaxError => error
  puts error.message
  p error.path
end
