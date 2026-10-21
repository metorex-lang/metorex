class Runner
  def self.load_uri
    require "uri"
  end
end

Runner.load_uri
p Object.autoload?(:IPSocket)
p Runner.autoload?(:IPSocket, false)
