def probe(value = :none, &block)
  [value, block ? :given : :no_block]
end

implementation = proc do |name, payload|
  p probe(name)
  p probe { :explicit }
  payload
end
p implementation.call(:load, 1) { :passed_along }
p implementation.yield(:save, 2) { :passed_along }
runner = lambda { |name, &inner| [probe(name), inner&.call] }
p runner.call(:run) { :inner }
def relay(*args, &block)
  yield(*args)
end
p(relay(:x) { |x| probe(x) })
