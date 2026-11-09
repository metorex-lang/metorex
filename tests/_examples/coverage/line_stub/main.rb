require "coverage"
p Coverage.line_stub(File.join(__dir__, "ledger.rb"))
require "tmpdir"
Dir.mktmpdir do |directory|
  config = File.join(directory, "database.yml")
  File.write(config, "development: &default\n  adapter: mysql2\n")
  begin
    Coverage.line_stub(config)
  rescue SyntaxError
    p :refused
  end
end
