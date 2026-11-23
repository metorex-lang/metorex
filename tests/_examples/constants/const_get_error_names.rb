# The name a NameError from Object.const_get carries, for names that are not
# constants or not spelled as one.
["", "::", "UnknownClass", "UnknownClass::Ace", "An invalid string", "InvalidClass\n",
 "blargle", "Ace::gas", "Object::Nope", "::Nope", :Nope, "Object::gas", "Comparable::Nope", "Object::", "Object::::Comparable", "Comparable::Missing::gas"].each do |name|
  Object.const_get(name)
rescue NameError => error
  p [name, error.name, error.message]
end
