attempts = 0
begin
  attempts += 1
  raise "fail" if attempts < 3
  puts "body succeeded on attempt #{attempts}"
rescue
  retry
ensure
  puts "statement ensure after #{attempts}"
end

tries = 0
value = begin
  tries += 1
  raise "fail" if tries < 2
  "value after #{tries}"
rescue
  retry
ensure
  puts "value ensure after #{tries}"
end
puts value

def fetch_with_retries
  calls = 0
  begin
    calls += 1
    raise "fail" if calls < 4
    "method answered after #{calls}"
  rescue
    retry
  ensure
    puts "method ensure after #{calls}"
  end
end
puts fetch_with_retries

def method_level
  @calls = (@calls || 0) + 1
  raise "fail" if @calls < 2
  "method-level rescue answered after #{@calls}"
rescue
  retry
ensure
  puts "method-level ensure after #{@calls}"
end
puts method_level
