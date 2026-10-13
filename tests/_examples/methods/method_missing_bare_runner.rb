module BarePredicates
  def method_missing(name, *arguments)
    return "predicate" if name.to_s.start_with?("be_")

    super
  end

  def respond_to_missing?(name, include_private = false)
    name.to_s.start_with?("be_") || super
  end
end
