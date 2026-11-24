module Counting
  def setting
    [:counted, super]
  end
end
parent = Class.new do
  class << self
    def setting
      :plain
    end
  end
end
child = Class.new(parent)
parent.singleton_class.prepend(Counting)
p parent.setting
p child.setting
p child.singleton_class.ancestors.first(3).map { |held| held == Counting ? :Counting : held.class }
