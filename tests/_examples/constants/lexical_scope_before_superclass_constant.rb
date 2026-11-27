# A class body finds a constant of the module it is written in before one
# its superclass has only registered as an autoload.
class Launcher
  autoload :Actions, File.join(__dir__, "lexical_scope_before_superclass_autoload", "launcher_actions")
end

module Shop
  module Actions
    def act
      :from_the_enclosing_module
    end
  end

  class Command < Launcher
    include Actions
  end
end

p Shop::Command.ancestors.take(3)
p Shop::Command.new.act
p Launcher.autoload?(:Actions).nil?

module Shop
  class Report < Launcher
    p Actions
  end
end
