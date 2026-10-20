# `public` on a singleton class names a method the object's class defines,
# including one the core library writes, such as Array#size.
list = [1, 2, 3]
list.singleton_class.send(:public, :size)
p list.size
p list.singleton_class.public_method_defined?(:size)

text = +"crate"
class << text
  private :upcase
  public :upcase
end
p text.upcase
