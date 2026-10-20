# A class method redefined with `define_method` on the singleton class, and
# made public again, is still one the class responds to.
class Gateway
  def self.connect = :real
end

Gateway.singleton_class.class_exec do
  define_method(:connect) { |*arguments, &block| :fake }
  ruby2_keywords(:connect) if Module.private_method_defined?(:ruby2_keywords)
  __send__(:public, :connect)
end
p Gateway.connect
p Gateway.respond_to?(:connect)
p Gateway.respond_to?(:connect, true)
