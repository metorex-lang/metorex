class Exchange
  def self.authenticate(...) = build(...).summary

  def self.build(client, mechanism, *args, sasl_ir: true, **kwargs, &block)
    new(client, mechanism, args, sasl_ir, kwargs, block)
  end

  def initialize(*parts)
    @parts = parts
  end

  def summary
    client, mechanism, args, sasl_ir, kwargs, block = @parts
    [client, mechanism, args, sasl_ir, kwargs, block&.call]
  end
end

class Adapter
  def authenticate(...) = Exchange.authenticate(:adapter, ...)
end

p Adapter.new.authenticate("PLAIN", "user", "secret", sasl_ir: :when_cached)
p Adapter.new.authenticate("EXTERNAL", sasl_ir: false, realm: "mail.example.com") { :answered }
p Adapter.new.authenticate("PLAIN")
