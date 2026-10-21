def make_error(message = "maintenance window moved")
  begin
    raise message
  rescue ::Exception => error
    error
  end
end

p make_error.message

module Billing
  class Error < StandardError; end

  def self.charge
    raise ::ArgumentError, "amount missing"
  rescue ::ArgumentError, ::Billing::Error => problem
    "rescued #{problem.class}"
  end
end

p Billing.charge

module Mailer
  class Exception < ::RuntimeError; end

  def self.deliver
    raise IOError, "connection reset"
  rescue ::Exception => problem
    "rescued #{problem.class} as ::Exception"
  end

  def self.which
    [::Exception, Exception]
  end
end

p Mailer.deliver
p Mailer.which
