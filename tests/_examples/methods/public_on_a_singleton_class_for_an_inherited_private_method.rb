# `public` on a singleton class makes a private method the object's class
# defines, or a module it includes defines, callable on that object alone.
module Forwarding
  private

  def forwarded_scheme
    "ws"
  end
end
class Request
  include Forwarding

  def call_scheme
    scheme
  end

  private

  def scheme
    "https"
  end
end

request = Request.new
request.singleton_class.send(:public, :scheme)
p request.scheme
begin
  Request.new.scheme
rescue NoMethodError => error
  p error.class
end

forwarded = Request.new
forwarded.singleton_class.send(:public, :forwarded_scheme)
p forwarded.forwarded_scheme
