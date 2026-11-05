# A parameter's default value can start on the line after its `=`.
module Classes
  RESERVED = "reserved"
end

def normalize(component, character_class=
    Classes::RESERVED,
    leave_encoded='')
  [component, character_class, leave_encoded]
end

def self.class_level(first, second =
                     2)
  first + second
end

p normalize("a")
p normalize("a", "b", "c")
p class_level(1)
