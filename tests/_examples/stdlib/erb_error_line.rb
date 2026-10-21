require "erb"

template = ERB.new("<%= raise 'foo' %>")
begin
  template.result
rescue RuntimeError => error
  puts error.backtrace.first
end

multiline = ERB.new("first\n<% raise 'bar' %>\n")
begin
  multiline.result
rescue RuntimeError => error
  puts error.backtrace.first
end
