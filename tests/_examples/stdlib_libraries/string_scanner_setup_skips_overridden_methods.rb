require "strscan"

class Tokenizer < StringScanner
  def initialize(text)
    @calls = []
    super
  end

  def reset
    @calls << :reset
    super
  end

  def string=(text)
    @calls << :string=
    super
  end

  attr_reader :calls
end

tokenizer = Tokenizer.new("select 1")
p tokenizer.calls
p tokenizer.scan(/\w+/)
