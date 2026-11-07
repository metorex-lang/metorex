module Speaking
  def say(word, **options)
    "#{word}#{options[:mark]}"
  end
  alias :s :say

  def say!(word, **options)
    say(word, **options, mark: "!")
  end
end

module Speaker
  extend Speaking
end

module Reversing
  def say(word, **options)
    super(word, **options).reverse
  end
  alias :s :say
end

copy = Speaker.dup
copy.extend Reversing
p copy.say("ab", mark: "?")
p copy.s("ab")
p copy.say!("ab")
p Speaker.say("ab")

module Joining
  def say(*words)
    words.first + words[1]
  end
end

other = Speaker.dup
p other.say("ab")
other.extend Joining
p other.say("ab", "cd", mark: "?")
