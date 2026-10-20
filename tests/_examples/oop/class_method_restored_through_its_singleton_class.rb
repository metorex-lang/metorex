# A class method replaced on the singleton class and put back from a stashed
# alias answers `respond_to?` and calls as it did before.
class Gateway
  def self.connect = :real
end

singleton = Gateway.singleton_class
singleton.send(:alias_method, :__stashed_connect, :connect)
singleton.send(:define_method, :connect) { :fake }
p Gateway.connect

singleton.send(:remove_method, :connect)
singleton.send(:remove_method, :connect) if singleton.method_defined?(:connect, false)
singleton.send(:alias_method, :connect, :__stashed_connect)
singleton.send(:remove_method, :__stashed_connect)
p Gateway.connect
p Gateway.respond_to?(:connect)
p Gateway.respond_to?(:connect, true)
p singleton.instance_methods(false)
