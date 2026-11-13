# `send` and `__send__` naming a reader with a Symbol, the way a lexer
# reads its tables off its class: a private reader on a class, a reader on
# an instance, one redefined to compute its answer, one a module's
# `method_missing` stands behind, and a `public_send` refused a private one.
class Tables
  class << self
    attr_accessor :keys
    private :keys, :keys=
  end
  self.keys = [3, 1, 2]

  attr_reader :count

  def initialize
    @count = 0
  end

  def bump
    @count += 1
  end
end

table = Tables.new
read = []
100.times do
  table.bump
  read << [Tables.send(:keys).size, table.__send__(:count)]
end
p read.last
p read.uniq.size

class Tables
  def count
    @count * 10
  end
end
p table.send(:count)

class Ghost
  def method_missing(name, *arguments)
    name == :keys ? :missing : super
  end

  def respond_to_missing?(name, include_private = false)
    name == :keys || super
  end
end
p Ghost.new.send(:keys)

begin
  Tables.public_send(:keys)
rescue NoMethodError => error
  p error.message
end
