def first_frame(source, line)
  eval(source, binding, "(template)", line)
rescue RuntimeError => error
  error.backtrace.first
end

puts first_frame("#coding:UTF-8\nraise 'second line'", 0)
puts first_frame("raise 'first line'", 0)
puts first_frame("raise 'from one'", 1)
puts first_frame("\n\nraise 'third line'", 10)
