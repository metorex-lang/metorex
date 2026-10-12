# A subclass of an exception class can give the class's own methods another
# name, as RubyGems names `SystemExit#status` `exit_code`.
class Leaving < SystemExit
  alias_method :exit_code, :status
  alias_method :clean?, :success?
end

leaving = Leaving.new(3, "closing the ledger")
p([leaving.exit_code, leaving.clean?, Leaving.new(0).clean?])
p(leaving.respond_to?(:exit_code))
p(Leaving.instance_method(:exit_code).name)
