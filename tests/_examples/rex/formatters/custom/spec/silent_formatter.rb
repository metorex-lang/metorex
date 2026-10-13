require "rspec/core/formatters/base_text_formatter"

class SilentFormatter < RSpec::Core::Formatters::BaseTextFormatter
  RSpec::Core::Formatters.register self, :dump_summary

  def dump_summary(summary)
    output.puts "silent: #{summary.totals_line}"
  end
end
