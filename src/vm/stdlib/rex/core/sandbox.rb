module Rex
  module Core
    module Sandbox
      def self.sandboxed
        orig_config  = Rex.configuration
        orig_world   = Rex.world
        orig_example = Rex.current_example

        Rex.configuration = Rex::Core::Configuration.new
        Rex.world         = Rex::Core::World.new(Rex.configuration)

        yield Rex.configuration
      ensure
        Rex.configuration   = orig_config
        Rex.world           = orig_world
        Rex.current_example = orig_example
      end
    end
  end
end
