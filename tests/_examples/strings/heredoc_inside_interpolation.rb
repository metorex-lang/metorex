# A heredoc opened inside an interpolation reads its body from the lines
# after the string, as simplecov's Rails sandbox spec does.
def read_file(name) = "# #{name}"
def write_file(name, text) = puts("#{name}:\n#{text}")
write_file("test/test_helper.rb", "#{read_file("test/test_helper.rb")}\n#{<<~RUBY}")
  class ActiveSupport::TestCase
    parallelize_teardown { SimpleCov.result }
  end
RUBY
p :after
text = "before #{<<~A} middle"
  heredoc line
A
p text
month = "2026-11"
note = "window: #{<<-NOTE.strip}" + " (#{month})"
  maintenance "moves" to 02:00 #{month} \\ done
  NOTE
p note
p __LINE__
