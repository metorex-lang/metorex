# A delegator sends a name Kernel keeps private, such as `puts` or
# `format`, on to the object behind it rather than answering it itself.
require "delegate"

class Printer
  def puts(text)
    [:printed, text]
  end

  def format(*)
    :formatted
  end
end

wrapped = SimpleDelegator.new(Printer.new)
p wrapped.puts("line")
p wrapped.__send__(:format, "%d", 1)
begin
  SimpleDelegator.new(1).__send__(:not_there)
rescue NoMethodError => error
  p error.class
end
