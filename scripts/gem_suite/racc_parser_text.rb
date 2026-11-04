# Writes racc's lib/racc/parser-text.rb, the runtime's source as a String,
# which its Rakefile builds before the tests run. Loading that Rakefile needs
# rake-compiler, so this repeats its `file 'lib/racc/parser-text.rb'` task.
# It runs in the copy of the gem a test file runs in.

text = File.read("lib/racc/parser.rb")
text.sub!(/\A# *frozen[-_]string[-_]literal:.*\n/, "")
text.gsub!(/^require '(.*)'$/) do
  lib = $1
  code = File.read("lib/#{lib}.rb")
  code.sub!(/\A(?:#.*\n)+/, "")
  %[unless $".find {|p| p.end_with?('/#{lib}.rb')}\n$".push "\#{__dir__}/#{lib}.rb"\n#{code}\nend\n]
rescue
  $&
end
File.open("lib/racc/parser-text.rb", "wb") do |io|
  io.write(<<-eorb)
module Racc
  PARSER_TEXT = <<'__end_of_file__'
#{text}
__end_of_file__
end
  eorb
end
