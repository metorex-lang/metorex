module Speaking
  def say(word) = "#{word}."
end

module Speaker
  extend Speaking
end

module Reversing
  def say(word) = super(word).reverse
end

Speaker.extend Reversing
p Speaker.say("ab")
p Speaker.singleton_class.ancestors.first(3)
copy = Module.new { extend Speaking }.dup
copy.extend Reversing
p copy.say("cd")
