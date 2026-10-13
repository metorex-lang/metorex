class Builder
  def include(item)
    [:included, item]
  end

  def run
    first = include(1).first
    from_brace_block = proc { include(2).last }.call
    from_lambda = lambda { include(3).size }.call
    [first, from_brace_block, from_lambda]
  end
end

p Builder.new.run
