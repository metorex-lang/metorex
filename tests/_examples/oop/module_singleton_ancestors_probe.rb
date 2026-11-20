module Plain; end
module SelfExt
  extend self
end
p Plain.singleton_class.ancestors
p SelfExt.singleton_class.ancestors
p SelfExt.singleton_class.superclass
