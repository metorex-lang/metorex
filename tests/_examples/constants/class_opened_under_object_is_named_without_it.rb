# A class or module opened as `Object::Name` is a top-level constant, and
# its name leaves Object out.
class Object::Person < ::Object
end

module Object::Helpers
end

eval "class Object::Visitor; end"

p Person.name
p Helpers.name
p Visitor.name
p Object.const_get(:Person).equal?(Person)
