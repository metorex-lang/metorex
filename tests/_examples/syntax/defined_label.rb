# defined? as a hash label and a keyword argument, as rubocop-ast's node map
# writes it.
def options(**given) = given

p({ defined?: :DefinedNode, def: :DefNode })
p options(defined?: 1)
p options defined?: 2
held = 3
p defined?(held)
