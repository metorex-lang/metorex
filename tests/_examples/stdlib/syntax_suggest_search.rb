# syntax_suggest finds the lines around a syntax error with prism.
require "syntax_suggest/api"

source = <<~RUBY
  # A program Ruby's grammar refuses, which is never run.
  puts "started"
  puts 1 2
RUBY

io = StringIO.new
# With no limit, a slow machine still finishes the search.
SyntaxSuggest.call(io: io, source: source, filename: "refused.rb", timeout: nil)
puts io.string
