class TallyFormatter
  RSpec::Core::Formatters.register self, :example_passed, :example_failed, :dump_summary

  def initialize(output)
    @output = output
    @passed = 0
    @failed = 0
  end

  def example_passed(notification)
    @passed += 1
    @output.puts "passed: #{notification.example.description}"
  end

  def example_failed(notification)
    @failed += 1
    @output.puts "failed: #{notification.example.description}"
  end

  def example_pending(_notification)
    @output.puts "never called, since the class did not register it"
  end

  def dump_summary(summary)
    @output.puts "#{@passed} passed, #{@failed} failed of #{summary.example_count}"
  end
end

RSpec.configure do |config|
  config.add_formatter TallyFormatter
end
