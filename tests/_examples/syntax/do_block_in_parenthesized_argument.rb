# A `do` block given to a call inside the parentheses of another call, which
# is itself an argument written without parentheses, as RuboCop's specs write
# `expect(lines).to eq(Array.new(count) do ... end.sort)`.
def eq(value) = value

p eq(
  Array.new(2) do |index|
    "example#{index}.rb"
  end.sort
)
p eq(
  %w[a b].map do |path|
    path.upcase
  end
)
