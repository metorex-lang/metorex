def fail_here(*)
  raise ArgumentError, "from the chain"
end

def line_of_failure
  yield
rescue ArgumentError => error
  error.backtrace_locations.find { |location| location.label.include?("block") || location.label == "<main>" }&.lineno
end

p(line_of_failure do
  [1, 2]
    .map { |value| value * 2 }
    .then { |values| fail_here(values) }
end)

p(line_of_failure do
  proc {
    :held
  }.call.then { fail_here }
end)

p(line_of_failure do
  fail_here(
    1,
    2
  )
end)

class Proc
  def fail_after(*)
    raise ArgumentError, "from the method the chain called"
  end
end

p(line_of_failure do
  proc {
    :held
  }.fail_after(:argument)
end)

p(line_of_failure do
  proc {
    :held
  }.fail_after proc { :other }
end)
