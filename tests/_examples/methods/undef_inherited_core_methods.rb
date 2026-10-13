class BlankSlate
  kept = [:__id__, :__send__, :object_id, :class, :respond_to?]
  instance_methods.each do |name|
    undef_method(name) unless kept.include?(name)
  end
end

p BlankSlate.instance_methods.sort
