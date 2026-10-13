require "stringio"

CAPTURED = StringIO.new

RSpec.configure do |config|
  config.add_formatter RSpec::Core::Formatters::DocumentationFormatter.new(CAPTURED)
  config.after(:suite) do
    puts "captured:"
    puts CAPTURED.string.lines.map { |line| "| #{line}" }.join
  end
end
