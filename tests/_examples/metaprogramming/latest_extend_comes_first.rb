module Helpers
  def label = "helpers"
end
module Command
  extend Helpers
  extend self
  def label = "command"
end
p Command.label, Command.singleton_class.ancestors.first(3)
module Other
  extend self
  extend Helpers
  def label = "other"
end
p Other.label
