def find(name)
  raise NameError, "missing #{name}" if name == "::Child"
  name
end

def klass(name)
  if name.start_with?("C")
    return find("::#{name}") rescue NameError
  end
  find("Nested::#{name}")
end

p klass("Child")
p klass("Parent")

def found
  return find("ok") rescue :never
end
p found

def value_rescued
  result = find("::Child") rescue :rescued
  result
end
p value_rescued
