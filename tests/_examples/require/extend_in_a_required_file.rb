# `extend` at the top of a file required from inside a method extends `main`,
# the `self` of every file's top level.
main = self

class Loader
  def load_greeting
    require File.join(__dir__, "extend_in_a_required_file", "greeting")
  end
end

Loader.new.load_greeting
puts main.instance_eval { greet }
p Loader.new.respond_to?(:greet)

module Again
  def again = "again"
end
Object.new.instance_exec { require File.join(__dir__, "extend_in_a_required_file", "again") }
puts main.instance_eval { again }
