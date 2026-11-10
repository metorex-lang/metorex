# A module extended after one was prepended to the singleton class answers
# ahead of a module extended before it, as Active Record's class methods
# do for its inherited hooks.
module Naming
  private
    def inherited(base)
      puts "Naming#inherited #{base}"
      super
    end
end
module Tracking
  private
    def inherited(subclass)
      puts "Tracking#inherited #{subclass}"
      super
    end
end
module Deprecation
  def verifier_secret = :secret
end
class Record
  extend Naming
  singleton_class.prepend(Deprecation)
  extend Tracking
end
class User < Record; end
p Record.method(:inherited).owner
