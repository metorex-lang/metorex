begin
  exit(1)
rescue SystemExit => error
  p error.respond_to?(:status)
  p error.public_send(:status)
  p error.method(:status).owner
  p error.method(:status).arity
  p error.method(:success?).parameters
end

class Shutdown < SystemExit; end
p Shutdown.new(2).method(:status).owner
p KeyError.new.method(:key).owner
