# A method passed as a block to map takes each element as its first
# argument and leaves its optional ones at their defaults, also when it is a
# module function reached through a block of its own.
module Unescaper
  def unescape(text, encoding = Encoding::UTF_8)
    [text, encoding]
  end
  module_function :unescape

  def parse(query, &unescaper)
    unescaper ||= method(:unescape)
    query.split("&").map { |pair| pair.split("=", 2).map(&unescaper) }
  end
  module_function :parse
end

p Unescaper.parse("a=1&b=2")
p ["x"].map(&Unescaper.method(:unescape))
p [["k", "v"]].map(&Unescaper.method(:unescape))
