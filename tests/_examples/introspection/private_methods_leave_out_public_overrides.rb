# A public method a class defines over one of Kernel's private functions is
# public, so `private_methods` leaves the name out, as Thor checks before it
# runs Bundler's `exec` command.
class Commands
  def exec(*arguments) = "running #{arguments.join(" ")}"
  def system = "reports"
end

commands = Commands.new
p commands.private_methods.include?(:exec)
p commands.private_methods.include?(:system)
p commands.private_methods.include?(:spawn)
p commands.public_methods.include?(:exec)
p commands.exec("totals.rb")
