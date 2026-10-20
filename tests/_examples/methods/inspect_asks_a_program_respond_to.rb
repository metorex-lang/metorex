# `inspect` asks a `respond_to?` the program wrote whether the object answers
# `instance_variables_to_inspect`, and leaves out what that method chooses
# when the answer is no.
class Crate
  def initialize
    @label = "fragile"
    @weight = 12
  end

  def respond_to?(name, include_private = false)
    puts "asked #{name.inspect}, #{include_private.inspect}"
    false
  end

  private

  def instance_variables_to_inspect = [:@label]
end

puts Crate.new.inspect.sub(/0x\h+/, "0x")
