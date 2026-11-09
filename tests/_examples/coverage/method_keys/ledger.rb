class Ledger
  def self.open
    new
  end

  define_method :balance do
    0
  end
end

module Auditing
  def self.included(base)
    base.class_eval do
      define_method(:audited?) { true }
    end
  end

  def audit
    :done
  end
end

Ledger.include Auditing
