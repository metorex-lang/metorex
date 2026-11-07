require "delegate"

class Entry
  def write(target, suppress: false, rewrite: false)
    [target, suppress, rewrite]
  end

  def options(settings)
    settings
  end

  def splatted(*values, **named)
    [values, named]
  end
end

class Wrapped < DelegateClass(Entry)
  def initialize
    super(Entry.new)
  end
end

wrapped = Wrapped.new
p wrapped.write(:io, suppress: true, rewrite: true)
p wrapped.write(:io)
p wrapped.options({ verbose: true })
p wrapped.splatted(1, 2, label: "x"), wrapped.splatted({ a: 1 })
p SimpleDelegator.new(Entry.new).write(:io, rewrite: true)
