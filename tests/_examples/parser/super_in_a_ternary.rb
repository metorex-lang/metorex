class Base
  def describe
    "base"
  end
end

class Derived < Base
  def describe
    defined?(super) ? super : "none"
  end

  def missing
    defined?(super) ? super : "none"
  end
end

puts Derived.new.describe
puts Derived.new.missing
