# A protected method of a class's singleton class is callable from a class
# method of one subclass on another subclass, as Thor's `subcommand` calls
# `subcommand_help`.
class Command
  class << self
    def register(name, handler)
      handler.describe(name)
    end

    protected

    def describe(name)
      "#{self.name} handles #{name}"
    end
  end
end

class Reports < Command
end

class Invoices < Command
  def self.wire = register("invoices", Reports)
end

p Invoices.wire
begin
  Reports.describe("outside")
rescue NoMethodError => error
  p error.message
end
