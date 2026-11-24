require "delegate"

class Wrapped < DelegateClass(Hash)
  def initialize
    # __setobj__ is not called yet.
  end
end

wrapped = Wrapped.allocate
p wrapped.respond_to?(:keys)
p wrapped.__getobj__ { :fallback }
begin
  wrapped.__getobj__
rescue ArgumentError => error
  p error.message
end
