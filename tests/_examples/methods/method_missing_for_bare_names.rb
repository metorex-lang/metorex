module Predicates
  def method_missing(name, *arguments)
    if name.to_s.start_with?("be_")
      "predicate #{name.to_s.delete_prefix('be_')}#{arguments.empty? ? '' : " #{arguments.inspect}"}"
    else
      super
    end
  end

  def respond_to_missing?(name, include_private = false)
    name.to_s.start_with?("be_") || super
  end
end

class Checker
  include Predicates

  def check = be_valid
end

checker = Checker.new
puts checker.check
puts checker.instance_exec { be_paid }
puts checker.instance_exec { be_within(3) }
puts Class.new(Checker).new.instance_exec { be_empty }

begin
  checker.instance_exec { not_a_predicate }
rescue NameError => error
  puts error.class
end
