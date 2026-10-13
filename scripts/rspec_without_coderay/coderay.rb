# On the load path of the RSpec runs scripts/rex_expected.sh makes, so
# RSpec's syntax highlighter finds no CodeRay and highlights nothing, as Rex
# does.
raise LoadError, "cannot load such file -- coderay"
