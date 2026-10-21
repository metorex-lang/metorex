# The bundled gems Ruby 4.0.7 ships, each required from what metorex carries
# and used once.
require "minitest"
p Minitest::Test.new("totals").assert_equal(2, 1 + 1)

require "test/unit/assertions"
p Test::Unit::Assertions.instance_method(:assert_equal).owner

require "power_assert"
p PowerAssert.respond_to?(:start)

require "rake"
p "src/ledger/entries.rb".pathmap("%n")

require "rexml/document"
p REXML::Document.new("<invoice><total>120</total></invoice>").root.elements["total"].text

require "rss"
p RSS::Utils.to_class_name("content_encoded")

require "net/imap"
p Net::IMAP.encode_utf7("Entwürfe")

require "net/pop"
p Net::POP3.default_port

require "net/smtp"
p Net::SMTP.default_port

require "rdoc"
p RDoc::Markup::ToHtml.new(RDoc::Options.new).convert("Due *2026-11-01*")

require "rbs"
p RBS::Parser.parse_type("Array[Integer]").to_s

require "typeprof"
p TypeProf::CodePosition.new(3, 4).to_s

require "repl_type_completor"
p ReplTypeCompletor.respond_to?(:analyze)

require "debug/session"
p DEBUGGER__.frame_depth.positive?

require "readline"
p Readline.equal?(Reline)
