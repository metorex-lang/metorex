class Association
  def self.define_callbacks(model)
    "parent #{model}"
  end

  def self.define_extensions
    "parent extensions"
  end

  private_class_method :define_callbacks, :define_extensions
end

class CollectionAssociation < Association
  def self.define_callbacks(model)
    "collection #{model}, then #{super}"
  end

  private_class_method :define_extensions
end

class HasMany < CollectionAssociation
end

p HasMany.define_callbacks("Post")
p CollectionAssociation.define_callbacks("Tag")
begin
  Association.define_callbacks("Comment")
rescue NoMethodError => error
  p error.message
end
begin
  HasMany.define_extensions
rescue NoMethodError => error
  p error.message
end
