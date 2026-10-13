module Kinds
  class << self
    def of(value)
      case value
      when Array then :array
      else :other
      end
    end
  end
end

class Host
  Array = :shadow
end

Host.class_exec do
  p Kinds.of([1])
end
