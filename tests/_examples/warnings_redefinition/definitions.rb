LIMIT = 1
LIMIT = 2
module Settings
  NAME = "a"
  NAME = "b"
end
Settings.const_set(:NAME, "c")
class Settings::Box; SIZE = 1; SIZE = 2; end
def greet = :one
def greet = :two
class Shape
  def area = 1
  alias_method :old_area, :area
  def area = 2
  def size = 1
end
class Shape
  def size = 2
end
