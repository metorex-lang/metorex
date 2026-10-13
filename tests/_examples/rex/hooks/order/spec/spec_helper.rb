$events = []

module Formatting
  def money(cents) = format("$%.2f", cents / 100.0)
end

module Counting
  def group_count = 3
end

module Loud
  def shout(text) = "#{text.upcase}!"
end

module Greeting
  def greet = "hello from #{self.class.name ? 'a group' : 'an anonymous group'}"
end

RSpec.configure do |config|
  config.include Formatting
  config.extend Counting, :counted
  config.prepend Greeting, :greeting
  config.include Loud, loud: true

  config.before(:suite) { $events << "suite before" }
  config.after(:suite) { $events << "suite after"; puts $events.join(", ") }
  config.before(:each) { $events << "config before each" }
  config.after(:each) { $events << "config after each" }
  config.before(:context) { $events << "config before context" }
  config.after(:context) { $events << "config after context" }
  config.around(:each) do |example|
    $events << "config around start"
    example.run
    $events << "config around end"
  end
end
