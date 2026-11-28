# `undef` in code a class runs through `instance_eval` undefines one of the
# class's own methods, from its singleton class, as `def` there defines one,
# and so it does in an `instance_eval` inside the class's `class_eval`.
class Account
  singleton_class.define_method(:active) { "every active account" }
  def self.closed = "every closed account"
  def self.archived = "every archived account"
  def self.pending = "every pending account"
  def active = "an active account"
end

Account.instance_eval "undef active"
Account.instance_eval { undef closed }
Account.class_eval { instance_eval "undef archived" }
Account.class_eval { instance_eval { undef pending } }

p Account.respond_to?(:active)
p Account.respond_to?(:closed)
p Account.respond_to?(:archived)
p Account.respond_to?(:pending)
p Account.new.active
