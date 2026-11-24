class Migration
  autoload :CommandRecorder, File.join(__dir__, "autoloaded_recorder/command_recorder")

  class Current < Migration
  end

  module Compatibility
    class V6_0 < Current
    end

    class V5_2 < V6_0
      module CommandRecorder
        def invert_transaction
          :inverted
        end
      end
    end

    class V5_1 < V5_2
    end
  end
end

p Migration::Compatibility::V5_2::CommandRecorder.class
p Migration::Compatibility::V5_2::CommandRecorder.instance_methods
p Migration.autoload?(:CommandRecorder).nil?
p Migration::CommandRecorder.class
p Migration::Compatibility.constants.sort
