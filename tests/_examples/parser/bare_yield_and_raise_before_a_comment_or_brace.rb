# A bare yield ends at a comment, and a bare raise ends at the brace that
# closes an interpolation.
def run_block
  begin
    yield # the block runs here
  rescue ArgumentError
    :rescued
  end
end

p run_block { :answered }
p run_block { raise ArgumentError }

value = "given"
puts "value=#{value || raise}"
begin
  missing = nil
  puts "missing=#{missing || raise}"
rescue RuntimeError => error
  p error.class
end
